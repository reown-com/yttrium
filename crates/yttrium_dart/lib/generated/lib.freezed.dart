// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'lib.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$Error {
  String get field0;

  /// Create a copy of Error
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $ErrorCopyWith<Error> get copyWith =>
      _$ErrorCopyWithImpl<Error>(this as Error, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Error &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'Error(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $ErrorCopyWith<$Res> {
  factory $ErrorCopyWith(Error value, $Res Function(Error) _then) =
      _$ErrorCopyWithImpl;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class _$ErrorCopyWithImpl<$Res> implements $ErrorCopyWith<$Res> {
  _$ErrorCopyWithImpl(this._self, this._then);

  final Error _self;
  final $Res Function(Error) _then;

  /// Create a copy of Error
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_self.copyWith(
      field0: null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [Error].
extension ErrorPatterns on Error {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Error_General value)? general,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case Error_General() when general != null:
        return general(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Error_General value) general,
  }) {
    final _that = this;
    switch (_that) {
      case Error_General():
        return general(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Error_General value)? general,
  }) {
    final _that = this;
    switch (_that) {
      case Error_General() when general != null:
        return general(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? general,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case Error_General() when general != null:
        return general(_that.field0);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) general,
  }) {
    final _that = this;
    switch (_that) {
      case Error_General():
        return general(_that.field0);
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? general,
  }) {
    final _that = this;
    switch (_that) {
      case Error_General() when general != null:
        return general(_that.field0);
      case _:
        return null;
    }
  }
}

/// @nodoc

class Error_General extends Error {
  const Error_General(this.field0) : super._();

  @override
  final String field0;

  /// Create a copy of Error
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $Error_GeneralCopyWith<Error_General> get copyWith =>
      _$Error_GeneralCopyWithImpl<Error_General>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Error_General &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'Error.general(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $Error_GeneralCopyWith<$Res>
    implements $ErrorCopyWith<$Res> {
  factory $Error_GeneralCopyWith(
          Error_General value, $Res Function(Error_General) _then) =
      _$Error_GeneralCopyWithImpl;
  @override
  @useResult
  $Res call({String field0});
}

/// @nodoc
class _$Error_GeneralCopyWithImpl<$Res>
    implements $Error_GeneralCopyWith<$Res> {
  _$Error_GeneralCopyWithImpl(this._self, this._then);

  final Error_General _self;
  final $Res Function(Error_General) _then;

  /// Create a copy of Error
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(Error_General(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$SolanaSignError {
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is SolanaSignError);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SolanaSignError()';
  }
}

/// @nodoc
class $SolanaSignErrorCopyWith<$Res> {
  $SolanaSignErrorCopyWith(
      SolanaSignError _, $Res Function(SolanaSignError) __);
}

/// Adds pattern-matching-related methods to [SolanaSignError].
extension SolanaSignErrorPatterns on SolanaSignError {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SolanaSignError_InvalidKeypair value)? invalidKeypair,
    TResult Function(SolanaSignError_InvalidTransaction value)?
        invalidTransaction,
    TResult Function(SolanaSignError_SignerNotRequired value)?
        signerNotRequired,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case SolanaSignError_InvalidKeypair() when invalidKeypair != null:
        return invalidKeypair(_that);
      case SolanaSignError_InvalidTransaction() when invalidTransaction != null:
        return invalidTransaction(_that);
      case SolanaSignError_SignerNotRequired() when signerNotRequired != null:
        return signerNotRequired(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SolanaSignError_InvalidKeypair value)
        invalidKeypair,
    required TResult Function(SolanaSignError_InvalidTransaction value)
        invalidTransaction,
    required TResult Function(SolanaSignError_SignerNotRequired value)
        signerNotRequired,
  }) {
    final _that = this;
    switch (_that) {
      case SolanaSignError_InvalidKeypair():
        return invalidKeypair(_that);
      case SolanaSignError_InvalidTransaction():
        return invalidTransaction(_that);
      case SolanaSignError_SignerNotRequired():
        return signerNotRequired(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SolanaSignError_InvalidKeypair value)? invalidKeypair,
    TResult? Function(SolanaSignError_InvalidTransaction value)?
        invalidTransaction,
    TResult? Function(SolanaSignError_SignerNotRequired value)?
        signerNotRequired,
  }) {
    final _that = this;
    switch (_that) {
      case SolanaSignError_InvalidKeypair() when invalidKeypair != null:
        return invalidKeypair(_that);
      case SolanaSignError_InvalidTransaction() when invalidTransaction != null:
        return invalidTransaction(_that);
      case SolanaSignError_SignerNotRequired() when signerNotRequired != null:
        return signerNotRequired(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? invalidKeypair,
    TResult Function(String field0)? invalidTransaction,
    TResult Function(String pubkey)? signerNotRequired,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case SolanaSignError_InvalidKeypair() when invalidKeypair != null:
        return invalidKeypair(_that.field0);
      case SolanaSignError_InvalidTransaction() when invalidTransaction != null:
        return invalidTransaction(_that.field0);
      case SolanaSignError_SignerNotRequired() when signerNotRequired != null:
        return signerNotRequired(_that.pubkey);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) invalidKeypair,
    required TResult Function(String field0) invalidTransaction,
    required TResult Function(String pubkey) signerNotRequired,
  }) {
    final _that = this;
    switch (_that) {
      case SolanaSignError_InvalidKeypair():
        return invalidKeypair(_that.field0);
      case SolanaSignError_InvalidTransaction():
        return invalidTransaction(_that.field0);
      case SolanaSignError_SignerNotRequired():
        return signerNotRequired(_that.pubkey);
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? invalidKeypair,
    TResult? Function(String field0)? invalidTransaction,
    TResult? Function(String pubkey)? signerNotRequired,
  }) {
    final _that = this;
    switch (_that) {
      case SolanaSignError_InvalidKeypair() when invalidKeypair != null:
        return invalidKeypair(_that.field0);
      case SolanaSignError_InvalidTransaction() when invalidTransaction != null:
        return invalidTransaction(_that.field0);
      case SolanaSignError_SignerNotRequired() when signerNotRequired != null:
        return signerNotRequired(_that.pubkey);
      case _:
        return null;
    }
  }
}

/// @nodoc

class SolanaSignError_InvalidKeypair extends SolanaSignError {
  const SolanaSignError_InvalidKeypair(this.field0) : super._();

  final String field0;

  /// Create a copy of SolanaSignError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $SolanaSignError_InvalidKeypairCopyWith<SolanaSignError_InvalidKeypair>
      get copyWith => _$SolanaSignError_InvalidKeypairCopyWithImpl<
          SolanaSignError_InvalidKeypair>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SolanaSignError_InvalidKeypair &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'SolanaSignError.invalidKeypair(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $SolanaSignError_InvalidKeypairCopyWith<$Res>
    implements $SolanaSignErrorCopyWith<$Res> {
  factory $SolanaSignError_InvalidKeypairCopyWith(
          SolanaSignError_InvalidKeypair value,
          $Res Function(SolanaSignError_InvalidKeypair) _then) =
      _$SolanaSignError_InvalidKeypairCopyWithImpl;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class _$SolanaSignError_InvalidKeypairCopyWithImpl<$Res>
    implements $SolanaSignError_InvalidKeypairCopyWith<$Res> {
  _$SolanaSignError_InvalidKeypairCopyWithImpl(this._self, this._then);

  final SolanaSignError_InvalidKeypair _self;
  final $Res Function(SolanaSignError_InvalidKeypair) _then;

  /// Create a copy of SolanaSignError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(SolanaSignError_InvalidKeypair(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class SolanaSignError_InvalidTransaction extends SolanaSignError {
  const SolanaSignError_InvalidTransaction(this.field0) : super._();

  final String field0;

  /// Create a copy of SolanaSignError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $SolanaSignError_InvalidTransactionCopyWith<
          SolanaSignError_InvalidTransaction>
      get copyWith => _$SolanaSignError_InvalidTransactionCopyWithImpl<
          SolanaSignError_InvalidTransaction>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SolanaSignError_InvalidTransaction &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'SolanaSignError.invalidTransaction(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $SolanaSignError_InvalidTransactionCopyWith<$Res>
    implements $SolanaSignErrorCopyWith<$Res> {
  factory $SolanaSignError_InvalidTransactionCopyWith(
          SolanaSignError_InvalidTransaction value,
          $Res Function(SolanaSignError_InvalidTransaction) _then) =
      _$SolanaSignError_InvalidTransactionCopyWithImpl;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class _$SolanaSignError_InvalidTransactionCopyWithImpl<$Res>
    implements $SolanaSignError_InvalidTransactionCopyWith<$Res> {
  _$SolanaSignError_InvalidTransactionCopyWithImpl(this._self, this._then);

  final SolanaSignError_InvalidTransaction _self;
  final $Res Function(SolanaSignError_InvalidTransaction) _then;

  /// Create a copy of SolanaSignError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(SolanaSignError_InvalidTransaction(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class SolanaSignError_SignerNotRequired extends SolanaSignError {
  const SolanaSignError_SignerNotRequired({required this.pubkey}) : super._();

  final String pubkey;

  /// Create a copy of SolanaSignError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $SolanaSignError_SignerNotRequiredCopyWith<SolanaSignError_SignerNotRequired>
      get copyWith => _$SolanaSignError_SignerNotRequiredCopyWithImpl<
          SolanaSignError_SignerNotRequired>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SolanaSignError_SignerNotRequired &&
            (identical(other.pubkey, pubkey) || other.pubkey == pubkey));
  }

  @override
  int get hashCode => Object.hash(runtimeType, pubkey);

  @override
  String toString() {
    return 'SolanaSignError.signerNotRequired(pubkey: $pubkey)';
  }
}

/// @nodoc
abstract mixin class $SolanaSignError_SignerNotRequiredCopyWith<$Res>
    implements $SolanaSignErrorCopyWith<$Res> {
  factory $SolanaSignError_SignerNotRequiredCopyWith(
          SolanaSignError_SignerNotRequired value,
          $Res Function(SolanaSignError_SignerNotRequired) _then) =
      _$SolanaSignError_SignerNotRequiredCopyWithImpl;
  @useResult
  $Res call({String pubkey});
}

/// @nodoc
class _$SolanaSignError_SignerNotRequiredCopyWithImpl<$Res>
    implements $SolanaSignError_SignerNotRequiredCopyWith<$Res> {
  _$SolanaSignError_SignerNotRequiredCopyWithImpl(this._self, this._then);

  final SolanaSignError_SignerNotRequired _self;
  final $Res Function(SolanaSignError_SignerNotRequired) _then;

  /// Create a copy of SolanaSignError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? pubkey = null,
  }) {
    return _then(SolanaSignError_SignerNotRequired(
      pubkey: null == pubkey
          ? _self.pubkey
          : pubkey // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

// dart format on
