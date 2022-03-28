with import <nixpkgs> {};
stdenv.mkDerivation rec {
        name = "fuck-jit-env";
        buildInputs = [ 
        pkgs.rustc
	    pkgs.cargo
        pkgs.clang
        pkgs.glib
        pkgs.zlib
        pkgs.llvmPackages_13.llvm.dev
        pkgs.libxml2.dev
        ];
     
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
        pkgs.zlib 
        pkgs.glib 
        pkgs.libxml2.dev 
    ];

    LLVM_SYS_130_PREFIX = pkgs.llvmPackages_13.llvm.dev;
    
}
