{ pkgs ? import <nixpkgs> {} }:
with pkgs;
rustPlatform.buildRustPackage rec {
  pname = "patchelf";
  version = builtins.readFile ./version;

  src = lib.cleanSource ./.;

  buildInputs = [ ];

  cargoSha256 = "1v3v8zgq4a7y9c13diz0hxb26a55s471n1zkr0ram0npzci8n3ns";
  verifyCargoDeps = true;

  meta = with stdenv.lib; {
    description = "A reimplementation of script in Rust";
    homepage = "https://github.com/NixOS/patchelf";
    license = licenses.gpl3;
    platforms = platforms.linux;
    maintainers = [ maintainers.fzakaria ];
  };
}
