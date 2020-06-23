{
pkgs ? import <nixpkgs> {}
}:

with pkgs;

stdenv.mkDerivation {
  name = "copernican";
  src = null;
  buildInputs = [ rustup gdb cgdb rr fuse pkgconfig ];

}
