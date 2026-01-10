package com.yttrium.wcpay

import android.content.Context

object YttriumWcpayKt {
    init {
        System.loadLibrary("uniffi_yttrium_wcpay")
    }

    external fun initializeTls(context: Context)
}

