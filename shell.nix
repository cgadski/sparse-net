let
  pkgs = import ./pkgs.nix {};
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
      # rust compiler and tools
      cargo
      rustfmt
      libiconv
      rustc
      rust-analyzer

      # graphviz + mars
      graphviz
      mars
    ];

    shellHook = ''
      PS1="sparse-net> "
    '';
  }
