# Fluyer

Build Core Project
```
cargo build -p fluyer_core
```

Build App
```
cmake -B ui-desktop/build -S ui-desktop -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
cmake --build ui-desktop/build
```

Run App
```
DYLD_LIBRARY_PATH="$PWD/libs/macos:$PWD/target/debug" ./ui-desktop/build/FluyerApp
```