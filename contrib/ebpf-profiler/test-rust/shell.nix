{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = [ pkgs.rustc pkgs.cargo ] ++ pkgs.bcc.nativeBuildInputs ;
  buildInputs = [ pkgs.bcc ];
}
