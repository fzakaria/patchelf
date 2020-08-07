let
  pkgs =
    import <nixpkgs> {};
  rust-toolchain = pkgs.symlinkJoin {
    name = "rust-toolchain";
    paths = [pkgs.rustc pkgs.cargo pkgs.rustPlatform.rustcSrc pkgs.rustfmt];
  };
in with pkgs;
mkShell {
  name = "patchelf";
  buildInputs = [rust-toolchain];
  RUST_BACKTRACE = 1;
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustcSrc}";
}