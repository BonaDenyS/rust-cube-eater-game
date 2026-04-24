UNAME := $(shell uname 2>/dev/null || echo Windows)

OUTPUT_DIR            = ./c_output
OPENGL_WRAPPER_LIB_DIR = ./opengl_wrapper_lib
OPENGL_WRAPPER_LIB_SRC = $(OPENGL_WRAPPER_LIB_DIR)/opengl_wrapper_lib.c
OPENGL_WRAPPER_LIB_OBJ = $(OUTPUT_DIR)/opengl_wrapper_lib.o
OPENGL_WRAPPER_LIB    = openglwrapper
TEST_GAME_DIR         = ./c_test_game
TEST_GAME_SRC         = $(TEST_GAME_DIR)/test_game.c

ifeq ($(UNAME), Linux)
  CARGO        = cargo
  GL_CFLAGS    =
  GL_LIBS      = -lglfw -lGL
  LIB_SO       = $(OUTPUT_DIR)/lib$(OPENGL_WRAPPER_LIB).so
  TEST_EXE     = $(OUTPUT_DIR)/test_game_exe
  BUILD_SO     = gcc -shared -fPIC -o $(LIB_SO) $(OPENGL_WRAPPER_LIB_OBJ) $(GL_LIBS)
  RUN_EXE      = LD_LIBRARY_PATH="$(OUTPUT_DIR):$$LD_LIBRARY_PATH" $(TEST_EXE)
  INSTALL_DEPS = sudo apt-get install -y gcc libglfw3-dev libgl1-mesa-dev

else ifeq ($(UNAME), Darwin)
  CARGO        = cargo
  BREW_PREFIX := $(shell brew --prefix 2>/dev/null || echo /usr/local)
  GL_CFLAGS    = -I$(BREW_PREFIX)/include -DGL_SILENCE_DEPRECATION
  GL_LIBS      = -L$(BREW_PREFIX)/lib -lglfw -framework OpenGL
  LIB_SO       = $(OUTPUT_DIR)/lib$(OPENGL_WRAPPER_LIB).dylib
  TEST_EXE     = $(OUTPUT_DIR)/test_game_exe
  BUILD_SO     = gcc -dynamiclib -o $(LIB_SO) $(OPENGL_WRAPPER_LIB_OBJ) $(GL_CFLAGS) $(GL_LIBS)
  RUN_EXE      = DYLD_LIBRARY_PATH="$(OUTPUT_DIR):$$DYLD_LIBRARY_PATH" $(TEST_EXE)
  INSTALL_DEPS = brew install glfw

else
  CARGO           = C:/Users/bona.suryana/.cargo/bin/cargo.exe
  export PATH    := C:/msys64/mingw64/bin:$(PATH)
  MINGW64_LIB     = C:/msys64/mingw64/lib
  MINGW64_INCLUDE = C:/msys64/mingw64/include
  GL_CFLAGS       = -I$(MINGW64_INCLUDE)
  GL_LIBS         = -L$(MINGW64_LIB) -lglfw3 -lopengl32
  LIB_SO          = $(OUTPUT_DIR)/lib$(OPENGL_WRAPPER_LIB).dll
  LIB_IMPLIB      = $(OUTPUT_DIR)/lib$(OPENGL_WRAPPER_LIB).dll.a
  TEST_EXE        = $(OUTPUT_DIR)/test_game_exe.exe
  BUILD_SO        = gcc -shared -o $(LIB_SO) $(OPENGL_WRAPPER_LIB_OBJ) $(GL_LIBS) -Wl,--out-implib,$(LIB_IMPLIB)
  RUN_EXE         = PATH="$(OUTPUT_DIR):$$PATH" $(TEST_EXE)
  INSTALL_DEPS    = @echo "Install MSYS2 from https://www.msys2.org then run: pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-glfw"
endif

.PHONY: install-deps
install-deps:
	$(INSTALL_DEPS)

.PHONY: build-c
build-c:
	@echo "Building OpenGL Wrapper library..."
	mkdir -p $(OUTPUT_DIR)
	gcc -c -fPIC $(GL_CFLAGS) $(OPENGL_WRAPPER_LIB_SRC) -o $(OPENGL_WRAPPER_LIB_OBJ)
	$(BUILD_SO)

.PHONY: run-c
run-c: build-c
	@echo "Running Test Game..."
	gcc $(GL_CFLAGS) $(TEST_GAME_SRC) -o $(TEST_EXE) -L$(OUTPUT_DIR) -l$(OPENGL_WRAPPER_LIB) $(GL_LIBS)
	$(RUN_EXE)

.PHONY: test-rust
test-rust:
	@echo "Running Rust Tests Serially..."
	$(CARGO) test --manifest-path ./my_game_engine/Cargo.toml tests::test_simple_game_loop -- --nocapture --ignored
	$(CARGO) test --manifest-path ./my_game_engine/Cargo.toml tests::test_sprite_rendering -- --nocapture --ignored
	$(CARGO) test --manifest-path ./my_game_engine/Cargo.toml tests::test_screen_clearing -- --nocapture --ignored
	$(CARGO) test --manifest-path ./my_game_engine/Cargo.toml tests::test_key_presses -- --nocapture --ignored
	$(CARGO) test --manifest-path ./my_game_engine/Cargo.toml tests::test_sprite_position_update -- --nocapture --ignored

.PHONY: run
run:
	@echo "Running Rust Test Game..."
	$(CARGO) run --manifest-path ./rust_test_game/Cargo.toml
