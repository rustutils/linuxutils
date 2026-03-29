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

      # Tool names — used for installing individual binaries and creating
      # multicall symlinks. Keep in sync with workspace members.
      tools = [
        "colrm"
        "line"
        "look"
        "mcookie"
        "mesg"
        "mountpoint"
        "nologin"
        "setpgid"
        "setsid"
        "rev"
        "uuidgen"
        "uuidparse"
      ];

      craneLibFor = system: crane.mkLib (pkgsFor system);

      # Source filter: keep Cargo/Rust files plus C headers for ABI compat.
      srcFor = system:
        let
          craneLib = craneLibFor system;
          headerFilter = path: _type: builtins.match ".*\\.h$" path != null;
          combinedFilter = path: type:
            (headerFilter path type) || (craneLib.filterCargoSources path type);
        in nixpkgs.lib.cleanSourceWith {
          src = ./.;
          filter = combinedFilter;
        };

      # Common args shared between deps and full builds.
      commonArgsFor = system: {
        src = srcFor system;
        strictDeps = true;
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

              install -m755 target/release/libuuid.so $out/lib/
              install -m644 libs/uuid/include/uuid.h $out/include/

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

                install -m755 target/release/libuuid.so $out/lib/
                install -m644 libs/uuid/include/uuid.h $out/include/

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
