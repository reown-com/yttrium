# WCPay-specific ProGuard rules
# These rules are in addition to the base proguard-rules.pro

# Keep generated uniffi bindings for wcpay
-keep class uniffi.uniffi_yttrium_wcpay.** { *; }
-keep class uniffi.yttrium_wcpay.** { *; }

# Preserve all classes that interact with JNI for wcpay
-keepclasseswithmembers class * {
    native <methods>;
}

# Preserve all Rust FFI bindings for wcpay
-keep class com.reown.yttrium.wcpay.** { *; }
-keep class com.yttrium.wcpay.** { *; }

# Preserve all org.rustls.platformverifier classes (used by wcpay)
-keep, includedescriptorclasses class org.rustls.platformverifier.** { *; }

# Optimize: Remove logging calls in release builds
-assumenosideeffects class android.util.Log {
    public static *** d(...);
    public static *** v(...);
    public static *** i(...);
}

# Keep native method registration
-keepclasseswithmembernames class * {
    native <methods>;
}
