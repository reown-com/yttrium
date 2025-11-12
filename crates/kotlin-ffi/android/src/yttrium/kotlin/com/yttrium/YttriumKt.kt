package com.yttrium

import android.content.Context

object YttriumKt {
    init {
        System.loadLibrary("uniffi_yttrium")
    }

    external fun initializeTls(context: Context)
}


