{ lib
, stdenv
, darwin
, fetchFromGitHub
, rustPlatform
, buildNpmPackage
}:
let
  pname = "sql-studio";
  version = "0.1.36";

  src = fetchFromGitHub {
    owner = "frectonz";
    repo = pname;
    rev = version;
    hash = "sha256-ZWGV4DYf+85LIGVDc8hcWSEJsM6UisuCB2Wd2kiw/sk=";
  };

  ui = buildNpmPackage {
    inherit version src;
    pname = "${pname}-ui";
    npmDepsHash = "sha256-/i3oEy/Jz5ge2oAOiqZFRB7cvCUItw+Z4l3VbK2aK2U=";
    sourceRoot = "${src.name}/ui";
    installPhase = ''
      cp -pr --reflink=auto -- dist "$out/"
    '';
  };
in
rustPlatform.buildRustPackage {
  inherit pname version src;

  useFetchCargoVendor = true;

  cargoHash = "sha256-rWG5iPXiG7kCf0yLAqcQi8AM3qv/WTUiY4cVrjpUc/Y=";

  preBuild = ''
    cp -pr --reflink=auto -- ${ui} ui/dist
  '';

  buildInputs = lib.optionals stdenv.isDarwin [ darwin.apple_sdk.frameworks.Foundation ];

  meta = {
    description = "SQL Database Explorer [SQLite, libSQL, PostgreSQL, MySQL/MariaDB, DuckDB, ClickHouse]";
    homepage = "https://github.com/frectonz/sql-studio";
    mainProgram = "sql-studio";
    license = lib.licenses.mit;
    maintainers = [ lib.maintainers.frectonz ];
    platforms = lib.platforms.all;
  };
}
