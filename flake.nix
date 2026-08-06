{
  description = "Rust Flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };

        chessrSnapshot = pkgs.writeShellApplication {
          name = "chessr-snapshot";
          runtimeInputs = [
            pkgs.git
            pkgs.coreutils
          ];
          text = ''
            # Usage: chessr-snapshot [name]
            # With no name, uses `git describe` — so if you've tagged the
            # commit (e.g. `git tag v0.2`) you get "chessr-v0.2". Untagged
            # commits fall back to a short hash; uncommitted changes get
            # a "-dirty" suffix so you never silently snapshot WIP.
            cargo build --profile release-fast
            name="''${1:-$(git describe --tags --always --dirty)}"
            mkdir -p baselines
            cp target/release-fast/chessr "baselines/chessr-$name"
            echo "Snapshot saved: baselines/chessr-$name"
          '';
        };
        chessrMatch = pkgs.writeShellApplication {
          name = "chessr-match";
          runtimeInputs = [
            pkgs.cutechess
            pkgs.findutils
          ];
          text = ''
            use_sprt=true
            pos_args=()
            for arg in "$@"; do
              if [ "$arg" = "--no-sprt" ]; then
                use_sprt=false
              else
                pos_args+=("$arg")
              fi
            done
            set -- "''${pos_args[@]}"

            if [ $# -lt 1 ]; then
              echo "Usage: chessr-match <old-version> [movetime_seconds=0.2] [book.pgn=books/8moves_v3.pgn] [--no-sprt]"
              exit 1
            fi
            spec="$1"
            st="''${2:-0.2}"
            book="''${3:-books/8moves_v3.pgn}"

            mkdir -p baselines

            if [ -f "$spec" ]; then
              old_bin="$spec"
            elif [ -f "baselines/chessr-$spec" ]; then
              old_bin="baselines/chessr-$spec"
            else
              echo "No binary found for '$spec' (checked '$spec' and 'baselines/chessr-$spec')" >&2
              echo "Available snapshots:" >&2
              mapfile -t snaps < <(find baselines -maxdepth 1 -type f -name 'chessr-*' -printf '  %f\n')
              if [ ''${#snaps[@]} -eq 0 ]; then
                echo "  (none yet — run chessr-snapshot)" >&2
              else
                printf '%s\n' "''${snaps[@]}" >&2
              fi
              exit 1
            fi

            cargo build --profile release-fast

            args=(
              -engine "cmd=$old_bin" "name=$spec"
              -engine cmd=target/release-fast/chessr name=new
              -each proto=uci "st=$st"
              -rounds 50 -repeat -games 2
              -pgnout results/vs_chessr_"$spec".pgn
              -concurrency 8
            )
            if [ "$use_sprt" = true ]; then
              args+=( -sprt elo0=0 elo1=10 alpha=0.05 beta=0.05 )
            fi
            if [ -n "$book" ]; then
              args+=( -openings "file=$book" format=pgn order=random )
            fi

            cutechess-cli "''${args[@]}"
          '';
        };
        chessrList = pkgs.writeShellApplication {
          name = "chessr-list";
          runtimeInputs = [ pkgs.findutils ];
          text = ''
            mkdir -p baselines
            mapfile -t snaps < <(find baselines -maxdepth 1 -type f -name 'chessr-*' -printf '%f\n' | sed 's/^chessr-//')
            if [ ''${#snaps[@]} -eq 0 ]; then
              echo "(no snapshots yet)"
            else
              printf '%s\n' "''${snaps[@]}"
            fi
          '';
        };
        chessrVsStockfish = pkgs.writeShellApplication {
          name = "chessr-vs-stockfish";
          runtimeInputs = [
            pkgs.cutechess
            pkgs.stockfish
          ];
          text = ''
            # Usage: chessr-vs-stockfish [elo=1500] [movetime_seconds=1/0.2] [book.pgn=books/8moves_v3.pgn]
            elo="''${1:-1500}"
            st="''${2:-1/0.2}"
            book="''${3:-books/8moves_v3.pgn}"

            cargo build --profile release-fast

            args=(
              -engine cmd=target/release-fast/chessr name=chessr
              -engine cmd=stockfish name=sf option.UCI_LimitStrength=true "option.UCI_Elo=$elo"
              -each proto=uci "tc=$st"
              -rounds 50 -repeat -games 2
              -pgnout results/vs_sf_"$elo".pgn
              -concurrency 8
            )
            if [ -n "$book" ]; then
              args+=( -openings "file=$book" format=pgn order=random )
            fi

            cutechess-cli "''${args[@]}"
          '';
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.rust-bin.stable.latest.complete
            pkgs.cargo-watch
            pkgs.perf
            pkgs.cutechess
            pkgs.stockfish
            chessrSnapshot
            chessrMatch
            chessrList
            chessrVsStockfish
          ];
          RUSTFLAGS = "-C target-cpu=native";
        };
      }
    );
}
