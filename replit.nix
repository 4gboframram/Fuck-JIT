 { pkgs ? import <nixpkgs> {} }: {
	deps = [
		pkgs.rustc
		pkgs.rustfmt
		pkgs.cargo
        
		pkgs.cargo-edit
        pkgs.rust-analyzer
        pkgs.rust-bindgen
        pkgs.clang
        pkgs.glib
        pkgs.zlib
        pkgs.llvmPackages_13.llvm.dev
        pkgs.libxml2.dev
	];
    env = {
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
      pkgs.zlib
      pkgs.glib
      pkgs.libxml2.dev
    ];
    
    LLVM_SYS_130_PREFIX = pkgs.llvmPackages_13.llvm.dev;

  };
}