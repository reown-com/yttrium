package com.yttrium

import android.content.Context

object YttriumKt {
    
    init {
        // Load the native library
        System.loadLibrary("uniffi_yttrium")
    }
    
    /**
     * Initialize rustls-platform-verifier for Android TLS support.
     * This must be called before making any HTTPS requests that use rustls.
     * 
     * @param context Android application context
     */
    external fun initializeTls(context: Context)
} 