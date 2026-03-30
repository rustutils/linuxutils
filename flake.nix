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

      # Library names — used for installing .so files and headers.
      # Keep in sync with libs/ workspace members.
      libs = [ "blkid" "fdisk" "mount" "smartcols" "uuid" ];

      # Tool names — used for installing individual binaries and creating
      # multicall symlinks. Keep in sync with workspace members.
      tools = [
        "cal"
        "choom"
        "chrt"
        "colrm"
        "flock"
        "hexdump"
        "ionice"
        "isosize"
        "line"
        "look"
        "lsirq"
        "lslocks"
        "lsmem"
        "lsns"
        "mcookie"
        "mesg"
        "namei"
        "mountpoint"
        "nologin"
        "prlimit"
        "rename"
        "renice"
        "setpgid"
        "setsid"
        "taskset"
        "rev"
        "rfkill"
        "uuidgen"
        "uuidparse"
        "waitpid"
        "whereis"
      ];

      craneLibFor = system: crane.mkLib (pkgsFor system);

      # Source filter: keep Cargo/Rust files plus C headers for ABI compat.
      srcFor = system:
        let
          craneLib = craneLibFor system;
          headerFilter = path: _type: builtins.match ".*\\.h(\\.in)?$" path != null;
          combinedFilter = path: type:
            (headerFilter path type) || (craneLib.filterCargoSources path type);
        in nixpkgs.lib.cleanSourceWith {
          src = ./.;
          filter = combinedFilter;
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
            cargo run --release --bin rustutils-mangen -- target/man
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
            pname = "rustutils-util-linux";

            installPhase = ''
              runHook preInstall

              mkdir -p $out/bin $out/lib $out/include $out/share/man/man1

              ${nixpkgs.lib.concatMapStringsSep "\n" (tool:
                "install -m755 target/release/${tool} $out/bin/"
              ) tools}

              ${nixpkgs.lib.concatMapStringsSep "\n" (lib:
                "install -m755 target/release/lib${lib}.so $out/lib/ || true"
              ) libs}

              for hdr in libs/*/include/*.h; do
                install -m644 "$hdr" $out/include/ || true
              done

              install -m644 target/man/*.1 $out/share/man/man1/

              runHook postInstall
            '';
          });

          # Multicall binary + libraries + headers + man pages.
          multicall = base.overrideAttrs (old: {
            pname = "rustutils-util-linux-multicall";

            installPhase = ''
                runHook preInstall

                mkdir -p $out/bin $out/lib $out/include $out/share/man/man1

                install -m755 target/release/rustutils-util-linux $out/bin/

                ${nixpkgs.lib.concatMapStringsSep "\n" (tool:
                  "ln -s $out/bin/rustutils-util-linux $out/bin/${tool}"
                ) tools}

                ${nixpkgs.lib.concatMapStringsSep "\n" (lib:
                  "install -m755 target/release/lib${lib}.so $out/lib/ || true"
                ) libs}

                for hdr in libs/*/include/*.h; do
                  install -m644 "$hdr" $out/include/ || true
                done

                install -m644 target/man/*.1 $out/share/man/man1/

                runHook postInstall
              '';
          });
        in {
          default = individual;
          rustutils-util-linux = individual;
          rustutils-util-linux-multicall = multicall;
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
