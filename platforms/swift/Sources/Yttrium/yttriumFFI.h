// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// The following structs are used to implement the lowest level
// of the FFI, and thus useful to multiple uniffied crates.
// We ensure they are declared exactly once, with a header guard, UNIFFI_SHARED_H.
#ifdef UNIFFI_SHARED_H
    // We also try to prevent mixing versions of shared uniffi header structs.
    // If you add anything to the #else block, you must increment the version suffix in UNIFFI_SHARED_HEADER_V4
    #ifndef UNIFFI_SHARED_HEADER_V4
        #error Combining helper code from multiple versions of uniffi is not supported
    #endif // ndef UNIFFI_SHARED_HEADER_V4
#else
#define UNIFFI_SHARED_H
#define UNIFFI_SHARED_HEADER_V4
// ⚠️ Attention: If you change this #else block (ending in `#endif // def UNIFFI_SHARED_H`) you *must* ⚠️
// ⚠️ increment the version suffix in all instances of UNIFFI_SHARED_HEADER_V4 in this file.           ⚠️

typedef struct RustBuffer
{
    uint64_t capacity;
    uint64_t len;
    uint8_t *_Nullable data;
} RustBuffer;

typedef struct ForeignBytes
{
    int32_t len;
    const uint8_t *_Nullable data;
} ForeignBytes;

// Error definitions
typedef struct RustCallStatus {
    int8_t code;
    RustBuffer errorBuf;
} RustCallStatus;

// ⚠️ Attention: If you change this #else block (ending in `#endif // def UNIFFI_SHARED_H`) you *must* ⚠️
// ⚠️ increment the version suffix in all instances of UNIFFI_SHARED_HEADER_V4 in this file.           ⚠️
#endif // def UNIFFI_SHARED_H
#ifndef UNIFFI_FFIDEF_RUST_FUTURE_CONTINUATION_CALLBACK
#define UNIFFI_FFIDEF_RUST_FUTURE_CONTINUATION_CALLBACK
typedef void (*UniffiRustFutureContinuationCallback)(uint64_t, int8_t
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_FREE
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_FREE
typedef void (*UniffiForeignFutureFree)(uint64_t
    );

#endif
#ifndef UNIFFI_FFIDEF_CALLBACK_INTERFACE_FREE
#define UNIFFI_FFIDEF_CALLBACK_INTERFACE_FREE
typedef void (*UniffiCallbackInterfaceFree)(uint64_t
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE
#define UNIFFI_FFIDEF_FOREIGN_FUTURE
typedef struct UniffiForeignFuture {
    uint64_t handle;
    UniffiForeignFutureFree _Nonnull free;
} UniffiForeignFuture;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_U8
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_U8
typedef struct UniffiForeignFutureStructU8 {
    uint8_t returnValue;
    RustCallStatus callStatus;
} UniffiForeignFutureStructU8;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_U8
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_U8
typedef void (*UniffiForeignFutureCompleteU8)(uint64_t, UniffiForeignFutureStructU8
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_I8
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_I8
typedef struct UniffiForeignFutureStructI8 {
    int8_t returnValue;
    RustCallStatus callStatus;
} UniffiForeignFutureStructI8;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_I8
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_I8
typedef void (*UniffiForeignFutureCompleteI8)(uint64_t, UniffiForeignFutureStructI8
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_U16
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_U16
typedef struct UniffiForeignFutureStructU16 {
    uint16_t returnValue;
    RustCallStatus callStatus;
} UniffiForeignFutureStructU16;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_U16
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_U16
typedef void (*UniffiForeignFutureCompleteU16)(uint64_t, UniffiForeignFutureStructU16
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_I16
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_I16
typedef struct UniffiForeignFutureStructI16 {
    int16_t returnValue;
    RustCallStatus callStatus;
} UniffiForeignFutureStructI16;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_I16
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_I16
typedef void (*UniffiForeignFutureCompleteI16)(uint64_t, UniffiForeignFutureStructI16
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_U32
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_U32
typedef struct UniffiForeignFutureStructU32 {
    uint32_t returnValue;
    RustCallStatus callStatus;
} UniffiForeignFutureStructU32;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_U32
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_U32
typedef void (*UniffiForeignFutureCompleteU32)(uint64_t, UniffiForeignFutureStructU32
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_I32
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_I32
typedef struct UniffiForeignFutureStructI32 {
    int32_t returnValue;
    RustCallStatus callStatus;
} UniffiForeignFutureStructI32;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_I32
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_I32
typedef void (*UniffiForeignFutureCompleteI32)(uint64_t, UniffiForeignFutureStructI32
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_U64
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_U64
typedef struct UniffiForeignFutureStructU64 {
    uint64_t returnValue;
    RustCallStatus callStatus;
} UniffiForeignFutureStructU64;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_U64
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_U64
typedef void (*UniffiForeignFutureCompleteU64)(uint64_t, UniffiForeignFutureStructU64
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_I64
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_I64
typedef struct UniffiForeignFutureStructI64 {
    int64_t returnValue;
    RustCallStatus callStatus;
} UniffiForeignFutureStructI64;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_I64
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_I64
typedef void (*UniffiForeignFutureCompleteI64)(uint64_t, UniffiForeignFutureStructI64
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_F32
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_F32
typedef struct UniffiForeignFutureStructF32 {
    float returnValue;
    RustCallStatus callStatus;
} UniffiForeignFutureStructF32;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_F32
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_F32
typedef void (*UniffiForeignFutureCompleteF32)(uint64_t, UniffiForeignFutureStructF32
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_F64
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_F64
typedef struct UniffiForeignFutureStructF64 {
    double returnValue;
    RustCallStatus callStatus;
} UniffiForeignFutureStructF64;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_F64
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_F64
typedef void (*UniffiForeignFutureCompleteF64)(uint64_t, UniffiForeignFutureStructF64
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_POINTER
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_POINTER
typedef struct UniffiForeignFutureStructPointer {
    void*_Nonnull returnValue;
    RustCallStatus callStatus;
} UniffiForeignFutureStructPointer;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_POINTER
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_POINTER
typedef void (*UniffiForeignFutureCompletePointer)(uint64_t, UniffiForeignFutureStructPointer
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_RUST_BUFFER
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_RUST_BUFFER
typedef struct UniffiForeignFutureStructRustBuffer {
    RustBuffer returnValue;
    RustCallStatus callStatus;
} UniffiForeignFutureStructRustBuffer;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_RUST_BUFFER
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_RUST_BUFFER
typedef void (*UniffiForeignFutureCompleteRustBuffer)(uint64_t, UniffiForeignFutureStructRustBuffer
    );

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_VOID
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_STRUCT_VOID
typedef struct UniffiForeignFutureStructVoid {
    RustCallStatus callStatus;
} UniffiForeignFutureStructVoid;

#endif
#ifndef UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_VOID
#define UNIFFI_FFIDEF_FOREIGN_FUTURE_COMPLETE_VOID
typedef void (*UniffiForeignFutureCompleteVoid)(uint64_t, UniffiForeignFutureStructVoid
    );

#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_CLONE_ACCOUNTCLIENT
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_CLONE_ACCOUNTCLIENT
void*_Nonnull uniffi_yttrium_fn_clone_accountclient(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_FREE_ACCOUNTCLIENT
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_FREE_ACCOUNTCLIENT
void uniffi_yttrium_fn_free_accountclient(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_CLONE_CLIENT
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_CLONE_CLIENT
void*_Nonnull uniffi_yttrium_fn_clone_client(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_FREE_CLIENT
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_FREE_CLIENT
void uniffi_yttrium_fn_free_client(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_CONSTRUCTOR_CLIENT_NEW
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_CONSTRUCTOR_CLIENT_NEW
void*_Nonnull uniffi_yttrium_fn_constructor_client_new(RustBuffer project_id, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_CREATE_SPONSORED_USER_OP
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_CREATE_SPONSORED_USER_OP
uint64_t uniffi_yttrium_fn_method_client_create_sponsored_user_op(void*_Nonnull ptr, RustBuffer chain_id, RustBuffer from, RustBuffer calls
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_PREPARE
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_PREPARE
uint64_t uniffi_yttrium_fn_method_client_prepare(void*_Nonnull ptr, RustBuffer chain_id, RustBuffer from, RustBuffer calls
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_PREPARE_DEPLOY
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_PREPARE_DEPLOY
uint64_t uniffi_yttrium_fn_method_client_prepare_deploy(void*_Nonnull ptr, RustBuffer auth_sig, RustBuffer params, RustBuffer sponsor
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_PREPARE_USDC_TRANSFER_CALL
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_PREPARE_USDC_TRANSFER_CALL
RustBuffer uniffi_yttrium_fn_method_client_prepare_usdc_transfer_call(void*_Nonnull ptr, RustBuffer chain_id, RustBuffer to, RustBuffer usdc_amount, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_SEND
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_SEND
uint64_t uniffi_yttrium_fn_method_client_send(void*_Nonnull ptr, RustBuffer signature, RustBuffer params
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_WITH_4337_URLS
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_WITH_4337_URLS
void*_Nonnull uniffi_yttrium_fn_method_client_with_4337_urls(void*_Nonnull ptr, RustBuffer bundler_url, RustBuffer paymaster_url, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_WITH_RPC_OVERRIDES
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_CLIENT_WITH_RPC_OVERRIDES
void*_Nonnull uniffi_yttrium_fn_method_client_with_rpc_overrides(void*_Nonnull ptr, RustBuffer rpc_overrides, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_CLONE_ERC6492CLIENT
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_CLONE_ERC6492CLIENT
void*_Nonnull uniffi_yttrium_fn_clone_erc6492client(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_FREE_ERC6492CLIENT
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_FREE_ERC6492CLIENT
void uniffi_yttrium_fn_free_erc6492client(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_CONSTRUCTOR_ERC6492CLIENT_NEW
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_CONSTRUCTOR_ERC6492CLIENT_NEW
void*_Nonnull uniffi_yttrium_fn_constructor_erc6492client_new(RustBuffer rpc_url, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_ERC6492CLIENT_VERIFY_SIGNATURE
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_METHOD_ERC6492CLIENT_VERIFY_SIGNATURE
uint64_t uniffi_yttrium_fn_method_erc6492client_verify_signature(void*_Nonnull ptr, RustBuffer signature, RustBuffer address, RustBuffer message_hash
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_FUNC_FUNDING_METADATA_TO_AMOUNT
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_FUNC_FUNDING_METADATA_TO_AMOUNT
RustBuffer uniffi_yttrium_fn_func_funding_metadata_to_amount(RustBuffer value, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_FUNC_FUNDING_METADATA_TO_BRIDGING_FEE_AMOUNT
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_FN_FUNC_FUNDING_METADATA_TO_BRIDGING_FEE_AMOUNT
RustBuffer uniffi_yttrium_fn_func_funding_metadata_to_bridging_fee_amount(RustBuffer value, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUSTBUFFER_ALLOC
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUSTBUFFER_ALLOC
RustBuffer ffi_yttrium_rustbuffer_alloc(uint64_t size, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUSTBUFFER_FROM_BYTES
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUSTBUFFER_FROM_BYTES
RustBuffer ffi_yttrium_rustbuffer_from_bytes(ForeignBytes bytes, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUSTBUFFER_FREE
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUSTBUFFER_FREE
void ffi_yttrium_rustbuffer_free(RustBuffer buf, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUSTBUFFER_RESERVE
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUSTBUFFER_RESERVE
RustBuffer ffi_yttrium_rustbuffer_reserve(RustBuffer buf, uint64_t additional, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_U8
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_U8
void ffi_yttrium_rust_future_poll_u8(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_U8
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_U8
void ffi_yttrium_rust_future_cancel_u8(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_U8
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_U8
void ffi_yttrium_rust_future_free_u8(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_U8
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_U8
uint8_t ffi_yttrium_rust_future_complete_u8(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_I8
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_I8
void ffi_yttrium_rust_future_poll_i8(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_I8
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_I8
void ffi_yttrium_rust_future_cancel_i8(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_I8
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_I8
void ffi_yttrium_rust_future_free_i8(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_I8
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_I8
int8_t ffi_yttrium_rust_future_complete_i8(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_U16
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_U16
void ffi_yttrium_rust_future_poll_u16(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_U16
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_U16
void ffi_yttrium_rust_future_cancel_u16(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_U16
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_U16
void ffi_yttrium_rust_future_free_u16(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_U16
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_U16
uint16_t ffi_yttrium_rust_future_complete_u16(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_I16
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_I16
void ffi_yttrium_rust_future_poll_i16(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_I16
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_I16
void ffi_yttrium_rust_future_cancel_i16(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_I16
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_I16
void ffi_yttrium_rust_future_free_i16(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_I16
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_I16
int16_t ffi_yttrium_rust_future_complete_i16(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_U32
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_U32
void ffi_yttrium_rust_future_poll_u32(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_U32
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_U32
void ffi_yttrium_rust_future_cancel_u32(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_U32
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_U32
void ffi_yttrium_rust_future_free_u32(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_U32
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_U32
uint32_t ffi_yttrium_rust_future_complete_u32(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_I32
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_I32
void ffi_yttrium_rust_future_poll_i32(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_I32
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_I32
void ffi_yttrium_rust_future_cancel_i32(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_I32
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_I32
void ffi_yttrium_rust_future_free_i32(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_I32
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_I32
int32_t ffi_yttrium_rust_future_complete_i32(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_U64
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_U64
void ffi_yttrium_rust_future_poll_u64(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_U64
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_U64
void ffi_yttrium_rust_future_cancel_u64(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_U64
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_U64
void ffi_yttrium_rust_future_free_u64(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_U64
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_U64
uint64_t ffi_yttrium_rust_future_complete_u64(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_I64
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_I64
void ffi_yttrium_rust_future_poll_i64(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_I64
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_I64
void ffi_yttrium_rust_future_cancel_i64(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_I64
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_I64
void ffi_yttrium_rust_future_free_i64(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_I64
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_I64
int64_t ffi_yttrium_rust_future_complete_i64(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_F32
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_F32
void ffi_yttrium_rust_future_poll_f32(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_F32
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_F32
void ffi_yttrium_rust_future_cancel_f32(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_F32
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_F32
void ffi_yttrium_rust_future_free_f32(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_F32
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_F32
float ffi_yttrium_rust_future_complete_f32(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_F64
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_F64
void ffi_yttrium_rust_future_poll_f64(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_F64
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_F64
void ffi_yttrium_rust_future_cancel_f64(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_F64
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_F64
void ffi_yttrium_rust_future_free_f64(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_F64
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_F64
double ffi_yttrium_rust_future_complete_f64(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_POINTER
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_POINTER
void ffi_yttrium_rust_future_poll_pointer(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_POINTER
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_POINTER
void ffi_yttrium_rust_future_cancel_pointer(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_POINTER
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_POINTER
void ffi_yttrium_rust_future_free_pointer(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_POINTER
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_POINTER
void*_Nonnull ffi_yttrium_rust_future_complete_pointer(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_RUST_BUFFER
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_RUST_BUFFER
void ffi_yttrium_rust_future_poll_rust_buffer(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_RUST_BUFFER
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_RUST_BUFFER
void ffi_yttrium_rust_future_cancel_rust_buffer(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_RUST_BUFFER
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_RUST_BUFFER
void ffi_yttrium_rust_future_free_rust_buffer(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_RUST_BUFFER
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_RUST_BUFFER
RustBuffer ffi_yttrium_rust_future_complete_rust_buffer(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_VOID
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_POLL_VOID
void ffi_yttrium_rust_future_poll_void(uint64_t handle, UniffiRustFutureContinuationCallback _Nonnull callback, uint64_t callback_data
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_VOID
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_CANCEL_VOID
void ffi_yttrium_rust_future_cancel_void(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_VOID
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_FREE_VOID
void ffi_yttrium_rust_future_free_void(uint64_t handle
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_VOID
#define UNIFFI_FFIDEF_FFI_YTTRIUM_RUST_FUTURE_COMPLETE_VOID
void ffi_yttrium_rust_future_complete_void(uint64_t handle, RustCallStatus *_Nonnull out_status
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_FUNC_FUNDING_METADATA_TO_AMOUNT
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_FUNC_FUNDING_METADATA_TO_AMOUNT
uint16_t uniffi_yttrium_checksum_func_funding_metadata_to_amount(void
    
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_FUNC_FUNDING_METADATA_TO_BRIDGING_FEE_AMOUNT
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_FUNC_FUNDING_METADATA_TO_BRIDGING_FEE_AMOUNT
uint16_t uniffi_yttrium_checksum_func_funding_metadata_to_bridging_fee_amount(void
    
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_CREATE_SPONSORED_USER_OP
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_CREATE_SPONSORED_USER_OP
uint16_t uniffi_yttrium_checksum_method_client_create_sponsored_user_op(void
    
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_PREPARE
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_PREPARE
uint16_t uniffi_yttrium_checksum_method_client_prepare(void
    
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_PREPARE_DEPLOY
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_PREPARE_DEPLOY
uint16_t uniffi_yttrium_checksum_method_client_prepare_deploy(void
    
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_PREPARE_USDC_TRANSFER_CALL
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_PREPARE_USDC_TRANSFER_CALL
uint16_t uniffi_yttrium_checksum_method_client_prepare_usdc_transfer_call(void
    
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_SEND
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_SEND
uint16_t uniffi_yttrium_checksum_method_client_send(void
    
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_WITH_4337_URLS
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_WITH_4337_URLS
uint16_t uniffi_yttrium_checksum_method_client_with_4337_urls(void
    
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_WITH_RPC_OVERRIDES
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_CLIENT_WITH_RPC_OVERRIDES
uint16_t uniffi_yttrium_checksum_method_client_with_rpc_overrides(void
    
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_ERC6492CLIENT_VERIFY_SIGNATURE
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_METHOD_ERC6492CLIENT_VERIFY_SIGNATURE
uint16_t uniffi_yttrium_checksum_method_erc6492client_verify_signature(void
    
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_CONSTRUCTOR_CLIENT_NEW
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_CONSTRUCTOR_CLIENT_NEW
uint16_t uniffi_yttrium_checksum_constructor_client_new(void
    
);
#endif
#ifndef UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_CONSTRUCTOR_ERC6492CLIENT_NEW
#define UNIFFI_FFIDEF_UNIFFI_YTTRIUM_CHECKSUM_CONSTRUCTOR_ERC6492CLIENT_NEW
uint16_t uniffi_yttrium_checksum_constructor_erc6492client_new(void
    
);
#endif
#ifndef UNIFFI_FFIDEF_FFI_YTTRIUM_UNIFFI_CONTRACT_VERSION
#define UNIFFI_FFIDEF_FFI_YTTRIUM_UNIFFI_CONTRACT_VERSION
uint32_t ffi_yttrium_uniffi_contract_version(void
    
);
#endif

