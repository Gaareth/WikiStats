force=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --force)
      force=true
      shift
      ;;
    *)
      shift
      ;;
  esac
done

bump_package_version() {
  local package=$1

  if [[ -z "$package" ]]; then
    echo "Usage: bump_package_version <package-name>"
    return 1
  fi

  # TODO: check if there are unpublished commits which changed a file in the package directory
  # Skip diff check if force is true
  if ! $force; then
    # Check if there are changes in the package directory
    if git diff --quiet -- "./$package"; then
      echo "No changes in ./$package, skipping version bump."
      return 1
    fi
  else
    echo "Force mode enabled: skipping diff check for $package."
  fi

  echo "> Select version bump type for $package:"
  select bump in patch minor major; do
    if [[ -n "$bump" ]]; then
      cargo set-version --bump "$bump" -p "$package"
      break
    else
      echo "Invalid selection."
    fi
  done
  return 0
}

build_and_push() {
  local package=$1
  local image_name=$2

  if [[ -z "$package" || -z "$image_name" ]]; then
    echo "Usage: build_and_push_docker <package-name> <image-name>"
    return 1
  fi

  new_version=$(grep '^version =' "$package/Cargo.toml" | head -n1 | cut -d'"' -f2)

  docker build \
    -t "ghcr.io/gaareth/wiki-stats-$image_name:latest" \
    -t "ghcr.io/gaareth/wiki-stats-$image_name:$new_version" \
    . \
    --push
}

#build_and_push server sp-server

if bump_package_version server; then
  build_and_push server sp-server
else
  echo "Skipping server build (no changes or user cancelled)"
fi

if bump_package_version cli; then
  echo "CLI version bumped successfully"
else
  echo "Skipping CLI version bump (no changes or user cancelled)"
fi