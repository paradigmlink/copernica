{ pkgs ? import <nixpkgs> {} }:

with pkgs;

let
  pre-commit = writeShellScriptBin "pre-commit" ''
    # get the staged files
    s_files=$(${git}/bin/git diff --name-only --cached)

    # if a staged file contains the keyword, get it out of the staged list
    for s_file in ''${s_files};do
        if grep -q -E '@nocheckin' ''${s_file};then
            echo "WARNING: ''${s_file} contains the keyword"
            ${git}/bin/git reset ''${s_file}
        fi
    done

    # if there is not any staged file left, fail the commit, otherwise
    # an empty commit would be created.
    s_files=$(${git}/bin/git diff --name-only --cached)
    if [[ "''${s_files}" = "" ]];then
        echo "WARNING: nothing to commit"
        exit 1
    fi
    exit 0
  '';
  monitor = writeShellScriptBin "monitor" ''
    dot -Tpng ./monitor.dot > ./monitor.png
    nohup feh --auto-reload --quiet monitor.png &
    nohup inotifywait -e close_write,moved_to,create -m . |
        while read -r directory events filename; do
          if [ "$filename" = "monitor.dot" ]; then
            dot -Tpng ./monitor.dot > ./monitor.png
          fi
        done &
  '';
in
stdenv.mkDerivation {
  name = "copernica";
  src = null;
  buildInputs = [ rustup gdb pkgconfig pre-commit ripgrep ];
}
