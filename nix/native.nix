{lib, ...}: {
  options = {
    native = lib.mkOption {
      type = lib.types.str;
      default = "native";
    };
  };
}
