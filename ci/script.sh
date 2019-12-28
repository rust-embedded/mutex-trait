set -euxo pipefail

main() {
    if [ $TRAVIS_RUST_VERSION = nightly ]; then
        export RUSTFLAGS="-D warnings"
    fi

    cargo check --target $TARGET

    case $TARGET in
        thumbv*-none-eabi*)
            ;;

        x86_64-unknown-linux-gnu)
            cargo test --target $TARGET
            ;;
    esac

    if [ $TRAVIS_RUST_VERSION = nightly ]; then
        # Get the latest nightly with a working clippy
        rustup toolchain uninstall nightly
        rustup set profile default
        rustup default nightly
        rustup target add $TARGET
        cargo clippy --target $TARGET -- -D warnings
    fi
}

main
