#!/usr/bin/sh

# wget -nv -O - https://raw.githubusercontent.com/BonnyAD9/uamp/master/packages/script/install.sh | sh

set -e

_REPO="https://github.com/BonnyAD9/uamp.git"
_GIT=${GIT_BINARY-git}
_CARGO=${CARGO_BINARY-cargo}
_GZIP=${GZIP_BINARY-gzip}
_UAMP=${UAMP_PATH-"/usr/bin/uamp"}
_FORCE=${FORCE_INSTALL-no}
_CLEAN=${UAMP_CLEANUP-tmp}
_MAN_DIR=${MAN_DIRECTORY-/usr/share/man}

# Define colors
if [ -t 1 ]; then
    _RESET=`printf '\x1b[0m'`
    _ITALIC=`printf '\x1b[3m'`
    _DYELLOW=`printf '\x1b[33m'`
    _GRAY=`printf '\x1b[90m'`
    _RED=`printf '\x1b[91m'`
    _GREEN=`printf '\x1b[92m'`
    _YELLOW=`printf '\x1b[93m'`
    _MAGENTA=`printf '\x1b[95m'`
    _CYAN=`printf '\x1b[96m'`
    _WHITE=`printf '\x1b[97m'`
    _SIGN=`printf '\x1b[38;2;250;50;170mB\x1b[38;2;240;50;180mo\x1b[38;2;230;50;190mn\x1b[38;2;220;50;200mn\x1b[38;2;210;50;210my\x1b[38;2;200;50;220mA\x1b[38;2;190;50;230mD\x1b[38;2;180;50;240m9'`
else
    _RESET=""
    _ITALIC=""
    _GRAY=""
    _RED=""
    _GREEN=""
    _MAGENTA=""
    _CYAN=""
    _WHITE=""
    _SIGN="BonnyAD9"
fi

error_out() {
    echo "${_RED}error:$_RESET $@"
    exit 1
}

_HELP="Welcome in help for the $_ITALIC${_GREEN}uamp installer$_RESET by $_SIGN$_RESET.
Version 0.1.0

This script is usually invoked directly from wget:
  wget -nv -O - https://raw.githubusercontent.com/BonnyAD9/uamp/master/packages/script/install.sh | sh

${_GREEN}sage:
  ${_CYAN}install.sh $_GRAY[${_DYELLOW}flags$_GRAY]$_RESET
    Install uamp.

${_GREEN}Flags:
  $_YELLOW-c  --cache $_WHITE<path>$_RESET
    Change path to temporary cache. This is by default new folder in /tmp.

  $_YELLOW-p  --path $_WHITE<path>$_RESET
    Path where uamp will be installed. $ITALIC/usr/bin/uamp$_RESET by default.

  $_YELLOW--force$_RESET
    Force installing even if uamp binary is already present. This will
    overwrite the old binary.

  $_YELLOW-r  --repository $_WHITE<url>$_RESET
    Choose repository from which uamp will be installed.

  $_YELLOW-t  --tag $_WHITE<tag>$_RESET
    Choose tag/commit/branch in repository which will be used to install. This
    is by default the latest tag. Use ${_ITALIC}master$_RESET to get the latest
    commit.

  $_YELLOW--git $_WHITE<git binary>$_RESET
    Choose git binary.

  $_YELLOW--cargo $_WHITE<cargo binary>$_RESET
    Choose cargo binary.

  $_YELLOW--clean $_WHITE(yes|tmp|no)$_RESET
    If set to ${_ITALIC}yes$_RESET the build directory will be removed after
    installation is complete. If set to ${_ITALIC}tmp$_RESET it will be deleted
    only if it wasn't given. Otherwise the directory will be left.

${_GREEN}Environment variables:$_RESET
  ${_MAGENTA}GIT_BINARY$_RESET
    Chooses the git binary. (${_ITALIC}git$_RESET by default)

  ${_MAGENTA}CARGO_BINARY$_RESET
    Chooses the cargo binary. (${_ITALIC}cargo$_RESET by default)

  ${_MAGENTA}GZIP_BINARY$_RESET
    Choose the gzip binary. (${_ITALIC}gzip$_RESET by default)

  ${_MAGENTA}UAMP_PATH$_RESET
    Choose uamp install path. (${_ITALIC}/usr/bin/uamp$_RESET by default)

  ${_MAGENTA}FORCE_INSTALL$_RESET
    Set to ${_ITALIC}yes$_RESET to force install uamp.

  ${_MAGENTA}MAN_DIRECTORY$_RESET
    Choose where manapges will be installed (${_ITALIC}/usr/share/man$_RESET by
    default).
"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|"-?"|--help)
            echo "$_HELP"
            exit 0
            ;;
        -p|--path)
            _UAMP="$2"
            shift && shift
            ;;
        -c|--cache)
            _DIR="$2"
            shift && shift
            ;;
        --force)
            _FORCE=yes
            shift
            ;;
        -r|--repository)
            _REPO="$2"
            shift && shift
            ;;
        -t|--tag)
            _TAG="$2"
            shift && shift
            ;;
        --git)
            _GIT="$2"
            shift && shift
            ;;
        --cargo)
            _CARGO="$2"
            shift && shift
            ;;
        --clean)
            _CLEAN="$2"
            shift && shift
            ;;
        *)
            error_out 'Unknown option `'"$1"'`.'
            ;;
    esac
done

# Check that this operation is possible
echo "Checking prerequisities..."

if ! type $_GIT &> /dev/null; then
    error_out "git is not installed."
fi

if ! type $_TAR &> /dev/null; then
    error_out "tar is not installed."
fi

if ! type $_CARGO &> /dev/null; then
    error_out 'cargo is not installed. See
https://doc.rust-lang.org/cargo/getting-started/installation.html for
installation instructions.'
fi

if [ -e "$_UAMP" ] && ! [ "$_FORCE" == yes ]; then
    error_out 'Uamp is already installed. Remove the binary at '"$_UAMP"' or
retry with `--force`.'
fi

echo "Creating temp directory..."

if [ -z "$_DIR" ]; then
    _DIR=`mktemp -d`
    if [ "$_CLEAN" == tmp ]; then
        _CLEAN=yes
    fi
    _CLEAN_DIR=$_DIR
fi

_WD=`pwd`

mkdir -p "$_DIR"
cd "$_DIR"

echo "Cloning uamp with git..."
$_GIT clone "$_REPO"
cd "`basename "$_REPO" .git`"
_GROOT=`pwd`

echo "Checking out..."
git fetch --tags
if [ -z "$_TAG" ]; then
    _TAG="`git describe --tag`"
fi
echo "checking out to $_ITALIC$_TAG$_RESET."
$_GIT checkout "$_TAG"

echo "Building uamp..."
UAMP_VERSION_COMMIT=`"$_GIT" rev-parse HEAD` $_CARGO build -r

echo "Building man pages..."
mkdir -p target/manpages
"$_GZIP" -c --best other/manpages/uamp.1 > target/manpages/uamp.1.gz
"$_GZIP" -c --best other/manpages/uamp.5 > target/manpages/uamp.5.gz

echo "Moving files to system..."
echo "Sudo is required for this final step."
sudo mkdir -p "`dirname "$_UAMP"`"
sudo mv "$_GROOT/target/release/uamp" "$_UAMP"
sudo mkdir -p /usr/man/man1
sudo mkdir -p /usr/man/man5
sudo mv "$_GROOT/target/manpages/uamp.1.gz" /usr/share/man/man1/uamp.1
sudo mv "$_GROOT/target/manpages/uamp.5.gz" /usr/share/man/man5/uamp.5
sudo mandb

echo "Checking that uamp works..."
if "$_UAMP" --version; then
    echo "${_GREEN}Success!$_RESET uamp is installed at $_ITALIC$_UAMP$_RESET!"
else
    error_out "uamp installation failed."
fi

if [ "$_CLEAN" == yes ]; then
    echo "Cleaning up..."
    if [ -z "$_CLEAN_DIR" ]; then
        _CLEAN_DIR="$_GROOT"
    fi
    rm -rf "${_CLEAN_DIR}"
fi
