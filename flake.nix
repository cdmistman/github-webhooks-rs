{
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

		devenv = {
			url = "github:cachix/devenv";
			inputs.nixpkgs.follows = "nixpkgs";
		};

		fenix = {
			url = "github:nix-community/fenix";
			inputs.nixpkgs.follows = "nixpkgs";
		};

		flake-parts.url = "github:hercules-ci/flake-parts";
	};

	outputs = inputs @ {
		devenv,
    fenix,
		flake-parts,
		nixpkgs,
		...
	}: flake-parts.lib.mkFlake { inherit inputs; } {
		imports = [
			devenv.flakeModule
		];

		systems = [
			"aarch64-darwin"
			"aarch64-linux"
			"x86_64-darwin"
			"x86_64-linux"
		];

		perSystem = { pkgs, system, ... }: {
      _module.args.pkgs = import nixpkgs {
        inherit system;

        overlays = [
          fenix.overlays.default
        ];
      };

			devenv.shells.default = {
        packages = pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.CoreFoundation
          pkgs.darwin.apple_sdk.frameworks.Security
        ];

				languages.rust = {
					enable = true;
					version = "latest";
				};

				scripts.download-schema.exec = ''
          ${pkgs.curl}/bin/curl \
            --proto '=https' \
            --tlsv1.2 \
            -sSfL \
            -o schema.json \
            https://unpkg.com/@octokit/webhooks-schemas/schema.json
				'';
			};
		};
	};
}
