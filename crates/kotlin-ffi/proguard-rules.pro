# Keep generated uniffi bindings
-keep class uniffi.** { *; }

# Preserve all classes that interact with JNI
-keepclasseswithmembers class * {
    native <methods>;
}

# Preserve all Rust FFI bindings
-keep class com.reown.yttrium.** { *; }