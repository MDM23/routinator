{
  outputs = { self, nixpkgs }:
    let
      pkgs = import nixpkgs { system = "x86_64-linux"; };
    in
    {
      formatter.x86_64-linux = (
        pkgs.nixpkgs-fmt
      );
      devShells.x86_64-linux.default = pkgs.mkShell {
        buildInputs = [
          pkgs.trunk
        ];
      };
    };
}
