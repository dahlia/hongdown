{
  lib,
  rustPlatform,
  fetchFromGitHub,
  ...
}:
rustPlatform.buildRustPackage rec {
  name = "hongdown";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "dahlia";
    repo = "hongdown";
    rev = "${version}";
    hash = "sha256-dXDzzXNy5noOVCTmzueospo9xJdwdDswAWpkkFBOeLQ=";
  };
  useFetchCargoVendor = true;
  cargoHash = "sha256-lV+lPM/AAnhvoekR5iEWbes9aShqTbkDCJSw56eqUgI=";
  meta = {
    description = "A Markdown formatter that enforces Hong Minhee's Markdown style conventions";
    mainProgram = "hongdown";
    homepage = "https://github.com/dahlia/hongdown";
    license = lib.licenses.gpl3;
    maintainers = [];
  };
}
