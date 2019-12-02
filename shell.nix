{ nixpkgs ? fetchTarball channel:nixos-unstable
, pkgs ? import nixpkgs {}
}:

with pkgs;

stdenv.mkDerivation {
  name = "copernican";

  src = null;

  buildInputs = [ rustup amp gdb ];

}
