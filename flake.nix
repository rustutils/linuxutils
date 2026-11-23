{
  description = "A collection of Linux utilities written in Rust, drop-in compatible with util-linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, crane }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      pkgsFor = system: nixpkgs.legacyPackages.${system};

      # Library names, used for installing the .so files and headers from
      # compat/lib<name>/. The tool names come from `linuxutils --list`.
      libs = [ "blkid" "fdisk" "mount" "smartcols" "uuid" ];

      craneLibFor = system: crane.mkLib (pkgsFor system);

      # The whole source tree except build output. The build also needs
      # files that a Cargo-only filter drops: C headers, the hand-written
      # man page sections, and the snapshots and fixtures the tests read.
      srcFor = system:
        nixpkgs.lib.cleanSourceWith {
          src = nixpkgs.lib.cleanSource ./.;
          filter = path: type:
            !(type == "directory" && baseNameOf path == "target");
        };

      # Common args shared between deps and full builds.
      commonArgsFor = system:
        let pkgs = pkgsFor system;
        in {
          src = srcFor system;
          strictDeps = true;
          nativeBuildInputs = [ pkgs.pkg-config pkgs.clang ];
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };

      # Build only workspace dependencies (cached separately from source).
      cargoArtifactsFor = system:
        let craneLib = craneLibFor system;
        in craneLib.buildDepsOnly (commonArgsFor system);

      # Full workspace build.
      buildBase = system:
        let
          pkgs = pkgsFor system;
          craneLib = craneLibFor system;
          commonArgs = commonArgsFor system;
        in craneLib.buildPackage (commonArgs // {
          cargoArtifacts = cargoArtifactsFor system;

          doCheck = true;
          nativeCheckInputs = [ pkgs.cargo-nextest pkgs.gcc ];
          checkPhase = ''
            cargo nextest run --release
          '';

          # Generate man pages during build.
          postBuild = ''
            cargo run --release --package xtask -- mangen target/man
          '';

          # Don't install anything yet — the specific packages below handle that.
          doInstallCargoArtifacts = false;
        });
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          base = buildBase system;

          # Individual binaries + libraries + headers + man pages.
          individual = base.overrideAttrs (old: {
            pname = "linuxutils";

            installPhase = ''
              runHook preInstall

              mkdir -p $out/bin $out/lib $out/include $out/share/man/man1

              for tool in $(target/release/linuxutils --list); do
                install -m755 "target/release/$tool" $out/bin/
              done

              ${nixpkgs.lib.concatMapStringsSep "\n" (lib:
                "install -m755 target/release/lib${lib}.so $out/lib/ || true"
              ) libs}

              ${nixpkgs.lib.concatMapStringsSep "\n" (lib:
                "install -m644 compat/lib${lib}/include/*.h $out/include/ || true"
              ) libs}

              install -m644 target/man/*.1 $out/share/man/man1/

              runHook postInstall
            '';
          });

          # Multicall binary + libraries + headers + man pages.
          multicall = base.overrideAttrs (old: {
            pname = "linuxutils-multicall";

            installPhase = ''
                runHook preInstall

                mkdir -p $out/bin $out/lib $out/include $out/share/man/man1

                install -m755 target/release/linuxutils $out/bin/

                for tool in $(target/release/linuxutils --list); do
                  ln -s $out/bin/linuxutils "$out/bin/$tool"
                done

                ${nixpkgs.lib.concatMapStringsSep "\n" (lib:
                  "install -m755 target/release/lib${lib}.so $out/lib/ || true"
                ) libs}

                ${nixpkgs.lib.concatMapStringsSep "\n" (lib:
                  "install -m644 compat/lib${lib}/include/*.h $out/include/ || true"
                ) libs}

                install -m644 target/man/*.1 $out/share/man/man1/

                runHook postInstall
              '';
          });
        in {
          default = individual;
          linuxutils = individual;
          linuxutils-multicall = multicall;
        }
      );

      devShells = forAllSystems (system:
        let pkgs = pkgsFor system;
        in {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo
              rustc
              clippy
              rustfmt
              cargo-nextest
            ];
          };
        }
      );
    };
}
