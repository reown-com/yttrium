package com.yttrium.utils

import android.content.Context

object YttriumUtilsKt {
    init {
        System.loadLibrary("uniffi_yttrium_utils")
    }

    external fun initializeTls(context: Context)
}


