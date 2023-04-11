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

		perSystem = _: {
			devenv.shells.default = {
				languages.rust = {
					enable = true;
					version = "latest";
				};

				scripts.update-schema.exec = ''
				'';
			};
		};
	};
}
