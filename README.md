# 📍nix-kunai

> *Yet another nix dependency manager - simple code, but robust handling.*

> [!WARNING]
> While `nix-kunai` is currently fully functional,
> it does *not* currently guarantee any stability in its JSON format.
> Hopefully this will change very soon, but until then, *caveat emptor.*

## Why?

If you're making your own derivations,
you likely know the pain of keeping your custom derivations up to date by hand -
having to manually update the version, hash, and everything.
This only gets worse as more and more packages are added.
Flakes somewhat help,
but they're limited by the fact that they only track entire branches,
making only updating when a new release is pushed impossible.

A dependency manager like this one resolves this problem
by creating a singular file that can be sourced by nix files,
and adds a single command that can update those dependencies automatically.
This simplifies the update process down from manually editing values by hand,
to running a single command.

### Why `nix-kunai` over other solutions?

The existing solutions
([`niv`](https://github.com/nmattia/niv),
[`npins`](https://github.com/andir/npins),
and [`yae`](https://github.com/Fuwn/yae))
are perfectly alright,
and you should use them if they work for you!

However, I've personally run into a few gotchas with all the above,
and there are some caveats with the way each of them handle updates.
Not wanting to bulldoze over these projects
(and, honestly, not fully understanding their codebases),
I've decided to throw my own hat into the pile.

### Goals

The primary goal of this project is to be a version-pinning solution
that **gracefully handles tag checking and artifact fetching.**

Existing solutions tend to have caveats when checking tags for a git repo,
especially when the intention is to grab a build artifact -
either the latest tag is grabbed without checking an artifact at all,
or the update will fail completely if the artifact is not found.
This causes problems when a new tag is pushed without associated build artifacts,
intentionally or otherwise.

Checking tags of a git repo can also be inconsistent -
without careful filtering,
an older tag may be assumed to be newer than older tags thanks to a missing prefix,
or a filter may be too strict and remove too many tags.
Of course, there's no true perfect solution for this,
but the assumptions made (see [Design](#design) below)
are intended to make this as intuitive as possible.

Asides from that, other goals include:

- Having a reasonably small code footprint.
- Having a simple and intuitive user experience.

### Okay, but why is it named `nix-kunai`?

A kunai can be used to pin things,
and `nix-kunai` can be used to pin the version of sources.

## Installation

You can easily try out `nix-kunai` on your system without installing anything
by running `nix run github:ProspectPyxis/nix-kunai`.

If you wish to permenantly install this program,
this repository provides a flake for doing so.
Import the flake as an input, and the package may be found at:
```nix
inputs.nix-kunai.packages.${pkgs.hostPlatform.system}.nix-kunai
```

## Usage

### Command Line

You can see detailed command help by running `nix-kunai --help`.

Below is an example on how to set up, add, update, and delete sources:

```sh
# Initialize a kunai.lock file
nix-kunai init

# Adds a nix-kunai source named `go-grip`, starting at version v0.3.0
# Note the `--tag-prefix v` flag, which will strip the leading "v" from fetched tags
nix-kunai add \
  --tag-prefix v \
  go-grip
  'https://github.com/chrishrb/go-grip/releases/download/v{version}/go-grip-v{version}-linux-amd64.tar.gz'
  0.3.0

# Adds a nix-kunai source named `nixpkgs`, tracking the branch nixos-unstable
# Note the `--update-scheme static` flag - this allows nix-kunai to update the hash without changing the version
# Also note the `--unpack` flag, since we're downloading a tarball
nix-kunai add \
  --update-scheme static \
  --unpack \
  nixpkgs \
  'https://github.com/NixOS/nixpkgs/archive/{version}.tar.gz' \
  nixos-unstable

# Update all sources
nix-kunai update

# Delete the nixpkgs source that was added earlier
nix-kunai delete nixpkgs
```

### In nix files

To use the `kunai.lock` file, simply import it as a JSON file:

```nix
kunai = builtins.fromJSON (builtins.readFile ./kunai.lock);

# Later, assuming go-grip is a source in the file:

version = kunai.go-grip.version;
hash = kunai.go-grip.hash;
```

## Design

As mentioned above in [Goals](#goals),
`nix-kunai` intends to handle tag checking and artifact fetching gracefully above all else.
To this end, the following design decisions were made:

- Git tags are sorted according to `git`'s built-in `--sort='v:refname'` option.
This works with Semantic Versioning and can handle more non-standard versioning schemes,
such as `x-y-z` where dashes are used in place of dots.
- When fetching and filtering tags,
it is assumed that the first character after the prefix,
or the first character of the tag if no prefix was defined,
**is a digit** (0-9).
This decision was made under the premise that nearly every repository
should be using a versioning scheme that starts with a number for its tags,
rather than using a letter to start (such as using `a1` as the version).
- When updating, `nix-kunai` will always internally keep track of the latest version tag,
but will only update the main `version` key of a source
if a matching artifact was fetched successfully.
This gracefully handles cases where a version tag is pushed
but no release artifacts are made alongside it
by keeping the version as it was to prevent breakage,
but also ensuring `nix-kunai` knows the latest version it fetched
to prevent unnecessary extra requests.

## Contributing

Pull requests are welcome!
For bigger pull requests, though, please make an issue first so it can be discussed.

Feature requests are also welcome,
but please keep in mind that one of the goals of this project is a small code footprint -
any feature requests that are considered "too big" may be rejected.

## License

This project is licensed under [GNU GPL 3.0 or later](LICENSE.md).
