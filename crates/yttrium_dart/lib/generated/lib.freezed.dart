// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'lib.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$FFIError {
  String get field0 => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) general,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? general,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? general,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FFIError_General value) general,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FFIError_General value)? general,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FFIError_General value)? general,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;

  /// Create a copy of FFIError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $FFIErrorCopyWith<FFIError> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FFIErrorCopyWith<$Res> {
  factory $FFIErrorCopyWith(FFIError value, $Res Function(FFIError) then) =
      _$FFIErrorCopyWithImpl<$Res, FFIError>;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class _$FFIErrorCopyWithImpl<$Res, $Val extends FFIError>
    implements $FFIErrorCopyWith<$Res> {
  _$FFIErrorCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FFIError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_value.copyWith(
      field0: null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$FFIError_GeneralImplCopyWith<$Res>
    implements $FFIErrorCopyWith<$Res> {
  factory _$$FFIError_GeneralImplCopyWith(_$FFIError_GeneralImpl value,
          $Res Function(_$FFIError_GeneralImpl) then) =
      __$$FFIError_GeneralImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String field0});
}

/// @nodoc
class __$$FFIError_GeneralImplCopyWithImpl<$Res>
    extends _$FFIErrorCopyWithImpl<$Res, _$FFIError_GeneralImpl>
    implements _$$FFIError_GeneralImplCopyWith<$Res> {
  __$$FFIError_GeneralImplCopyWithImpl(_$FFIError_GeneralImpl _value,
      $Res Function(_$FFIError_GeneralImpl) _then)
      : super(_value, _then);

  /// Create a copy of FFIError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$FFIError_GeneralImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FFIError_GeneralImpl extends FFIError_General {
  const _$FFIError_GeneralImpl(this.field0) : super._();

  @override
  final String field0;

  @override
  String toString() {
    return 'FFIError.general(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FFIError_GeneralImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of FFIError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FFIError_GeneralImplCopyWith<_$FFIError_GeneralImpl> get copyWith =>
      __$$FFIError_GeneralImplCopyWithImpl<_$FFIError_GeneralImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) general,
  }) {
    return general(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? general,
  }) {
    return general?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? general,
    required TResult orElse(),
  }) {
    if (general != null) {
      return general(field0);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FFIError_General value) general,
  }) {
    return general(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FFIError_General value)? general,
  }) {
    return general?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FFIError_General value)? general,
    required TResult orElse(),
  }) {
    if (general != null) {
      return general(this);
    }
    return orElse();
  }
}

abstract class FFIError_General extends FFIError {
  const factory FFIError_General(final String field0) = _$FFIError_GeneralImpl;
  const FFIError_General._() : super._();

  @override
  String get field0;

  /// Create a copy of FFIError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FFIError_GeneralImplCopyWith<_$FFIError_GeneralImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$FFIStatusResponseData {
  Object get field0 => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(FFIStatusResponsePending field0) pending,
    required TResult Function(FFIStatusResponseCompleted field0) completed,
    required TResult Function(FFIStatusResponseError field0) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(FFIStatusResponsePending field0)? pending,
    TResult? Function(FFIStatusResponseCompleted field0)? completed,
    TResult? Function(FFIStatusResponseError field0)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(FFIStatusResponsePending field0)? pending,
    TResult Function(FFIStatusResponseCompleted field0)? completed,
    TResult Function(FFIStatusResponseError field0)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FFIStatusResponseData_Pending value) pending,
    required TResult Function(FFIStatusResponseData_Completed value) completed,
    required TResult Function(FFIStatusResponseData_Error value) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FFIStatusResponseData_Pending value)? pending,
    TResult? Function(FFIStatusResponseData_Completed value)? completed,
    TResult? Function(FFIStatusResponseData_Error value)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FFIStatusResponseData_Pending value)? pending,
    TResult Function(FFIStatusResponseData_Completed value)? completed,
    TResult Function(FFIStatusResponseData_Error value)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FFIStatusResponseDataCopyWith<$Res> {
  factory $FFIStatusResponseDataCopyWith(FFIStatusResponseData value,
          $Res Function(FFIStatusResponseData) then) =
      _$FFIStatusResponseDataCopyWithImpl<$Res, FFIStatusResponseData>;
}

/// @nodoc
class _$FFIStatusResponseDataCopyWithImpl<$Res,
        $Val extends FFIStatusResponseData>
    implements $FFIStatusResponseDataCopyWith<$Res> {
  _$FFIStatusResponseDataCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FFIStatusResponseData
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$FFIStatusResponseData_PendingImplCopyWith<$Res> {
  factory _$$FFIStatusResponseData_PendingImplCopyWith(
          _$FFIStatusResponseData_PendingImpl value,
          $Res Function(_$FFIStatusResponseData_PendingImpl) then) =
      __$$FFIStatusResponseData_PendingImplCopyWithImpl<$Res>;
  @useResult
  $Res call({FFIStatusResponsePending field0});
}

/// @nodoc
class __$$FFIStatusResponseData_PendingImplCopyWithImpl<$Res>
    extends _$FFIStatusResponseDataCopyWithImpl<$Res,
        _$FFIStatusResponseData_PendingImpl>
    implements _$$FFIStatusResponseData_PendingImplCopyWith<$Res> {
  __$$FFIStatusResponseData_PendingImplCopyWithImpl(
      _$FFIStatusResponseData_PendingImpl _value,
      $Res Function(_$FFIStatusResponseData_PendingImpl) _then)
      : super(_value, _then);

  /// Create a copy of FFIStatusResponseData
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$FFIStatusResponseData_PendingImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as FFIStatusResponsePending,
    ));
  }
}

/// @nodoc

class _$FFIStatusResponseData_PendingImpl
    extends FFIStatusResponseData_Pending {
  const _$FFIStatusResponseData_PendingImpl(this.field0) : super._();

  @override
  final FFIStatusResponsePending field0;

  @override
  String toString() {
    return 'FFIStatusResponseData.pending(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FFIStatusResponseData_PendingImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of FFIStatusResponseData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FFIStatusResponseData_PendingImplCopyWith<
          _$FFIStatusResponseData_PendingImpl>
      get copyWith => __$$FFIStatusResponseData_PendingImplCopyWithImpl<
          _$FFIStatusResponseData_PendingImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(FFIStatusResponsePending field0) pending,
    required TResult Function(FFIStatusResponseCompleted field0) completed,
    required TResult Function(FFIStatusResponseError field0) error,
  }) {
    return pending(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(FFIStatusResponsePending field0)? pending,
    TResult? Function(FFIStatusResponseCompleted field0)? completed,
    TResult? Function(FFIStatusResponseError field0)? error,
  }) {
    return pending?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(FFIStatusResponsePending field0)? pending,
    TResult Function(FFIStatusResponseCompleted field0)? completed,
    TResult Function(FFIStatusResponseError field0)? error,
    required TResult orElse(),
  }) {
    if (pending != null) {
      return pending(field0);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FFIStatusResponseData_Pending value) pending,
    required TResult Function(FFIStatusResponseData_Completed value) completed,
    required TResult Function(FFIStatusResponseData_Error value) error,
  }) {
    return pending(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FFIStatusResponseData_Pending value)? pending,
    TResult? Function(FFIStatusResponseData_Completed value)? completed,
    TResult? Function(FFIStatusResponseData_Error value)? error,
  }) {
    return pending?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FFIStatusResponseData_Pending value)? pending,
    TResult Function(FFIStatusResponseData_Completed value)? completed,
    TResult Function(FFIStatusResponseData_Error value)? error,
    required TResult orElse(),
  }) {
    if (pending != null) {
      return pending(this);
    }
    return orElse();
  }
}

abstract class FFIStatusResponseData_Pending extends FFIStatusResponseData {
  const factory FFIStatusResponseData_Pending(
          final FFIStatusResponsePending field0) =
      _$FFIStatusResponseData_PendingImpl;
  const FFIStatusResponseData_Pending._() : super._();

  @override
  FFIStatusResponsePending get field0;

  /// Create a copy of FFIStatusResponseData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FFIStatusResponseData_PendingImplCopyWith<
          _$FFIStatusResponseData_PendingImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FFIStatusResponseData_CompletedImplCopyWith<$Res> {
  factory _$$FFIStatusResponseData_CompletedImplCopyWith(
          _$FFIStatusResponseData_CompletedImpl value,
          $Res Function(_$FFIStatusResponseData_CompletedImpl) then) =
      __$$FFIStatusResponseData_CompletedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({FFIStatusResponseCompleted field0});
}

/// @nodoc
class __$$FFIStatusResponseData_CompletedImplCopyWithImpl<$Res>
    extends _$FFIStatusResponseDataCopyWithImpl<$Res,
        _$FFIStatusResponseData_CompletedImpl>
    implements _$$FFIStatusResponseData_CompletedImplCopyWith<$Res> {
  __$$FFIStatusResponseData_CompletedImplCopyWithImpl(
      _$FFIStatusResponseData_CompletedImpl _value,
      $Res Function(_$FFIStatusResponseData_CompletedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FFIStatusResponseData
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$FFIStatusResponseData_CompletedImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as FFIStatusResponseCompleted,
    ));
  }
}

/// @nodoc

class _$FFIStatusResponseData_CompletedImpl
    extends FFIStatusResponseData_Completed {
  const _$FFIStatusResponseData_CompletedImpl(this.field0) : super._();

  @override
  final FFIStatusResponseCompleted field0;

  @override
  String toString() {
    return 'FFIStatusResponseData.completed(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FFIStatusResponseData_CompletedImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of FFIStatusResponseData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FFIStatusResponseData_CompletedImplCopyWith<
          _$FFIStatusResponseData_CompletedImpl>
      get copyWith => __$$FFIStatusResponseData_CompletedImplCopyWithImpl<
          _$FFIStatusResponseData_CompletedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(FFIStatusResponsePending field0) pending,
    required TResult Function(FFIStatusResponseCompleted field0) completed,
    required TResult Function(FFIStatusResponseError field0) error,
  }) {
    return completed(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(FFIStatusResponsePending field0)? pending,
    TResult? Function(FFIStatusResponseCompleted field0)? completed,
    TResult? Function(FFIStatusResponseError field0)? error,
  }) {
    return completed?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(FFIStatusResponsePending field0)? pending,
    TResult Function(FFIStatusResponseCompleted field0)? completed,
    TResult Function(FFIStatusResponseError field0)? error,
    required TResult orElse(),
  }) {
    if (completed != null) {
      return completed(field0);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FFIStatusResponseData_Pending value) pending,
    required TResult Function(FFIStatusResponseData_Completed value) completed,
    required TResult Function(FFIStatusResponseData_Error value) error,
  }) {
    return completed(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FFIStatusResponseData_Pending value)? pending,
    TResult? Function(FFIStatusResponseData_Completed value)? completed,
    TResult? Function(FFIStatusResponseData_Error value)? error,
  }) {
    return completed?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FFIStatusResponseData_Pending value)? pending,
    TResult Function(FFIStatusResponseData_Completed value)? completed,
    TResult Function(FFIStatusResponseData_Error value)? error,
    required TResult orElse(),
  }) {
    if (completed != null) {
      return completed(this);
    }
    return orElse();
  }
}

abstract class FFIStatusResponseData_Completed extends FFIStatusResponseData {
  const factory FFIStatusResponseData_Completed(
          final FFIStatusResponseCompleted field0) =
      _$FFIStatusResponseData_CompletedImpl;
  const FFIStatusResponseData_Completed._() : super._();

  @override
  FFIStatusResponseCompleted get field0;

  /// Create a copy of FFIStatusResponseData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FFIStatusResponseData_CompletedImplCopyWith<
          _$FFIStatusResponseData_CompletedImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FFIStatusResponseData_ErrorImplCopyWith<$Res> {
  factory _$$FFIStatusResponseData_ErrorImplCopyWith(
          _$FFIStatusResponseData_ErrorImpl value,
          $Res Function(_$FFIStatusResponseData_ErrorImpl) then) =
      __$$FFIStatusResponseData_ErrorImplCopyWithImpl<$Res>;
  @useResult
  $Res call({FFIStatusResponseError field0});
}

/// @nodoc
class __$$FFIStatusResponseData_ErrorImplCopyWithImpl<$Res>
    extends _$FFIStatusResponseDataCopyWithImpl<$Res,
        _$FFIStatusResponseData_ErrorImpl>
    implements _$$FFIStatusResponseData_ErrorImplCopyWith<$Res> {
  __$$FFIStatusResponseData_ErrorImplCopyWithImpl(
      _$FFIStatusResponseData_ErrorImpl _value,
      $Res Function(_$FFIStatusResponseData_ErrorImpl) _then)
      : super(_value, _then);

  /// Create a copy of FFIStatusResponseData
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$FFIStatusResponseData_ErrorImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as FFIStatusResponseError,
    ));
  }
}

/// @nodoc

class _$FFIStatusResponseData_ErrorImpl extends FFIStatusResponseData_Error {
  const _$FFIStatusResponseData_ErrorImpl(this.field0) : super._();

  @override
  final FFIStatusResponseError field0;

  @override
  String toString() {
    return 'FFIStatusResponseData.error(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FFIStatusResponseData_ErrorImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of FFIStatusResponseData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FFIStatusResponseData_ErrorImplCopyWith<_$FFIStatusResponseData_ErrorImpl>
      get copyWith => __$$FFIStatusResponseData_ErrorImplCopyWithImpl<
          _$FFIStatusResponseData_ErrorImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(FFIStatusResponsePending field0) pending,
    required TResult Function(FFIStatusResponseCompleted field0) completed,
    required TResult Function(FFIStatusResponseError field0) error,
  }) {
    return error(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(FFIStatusResponsePending field0)? pending,
    TResult? Function(FFIStatusResponseCompleted field0)? completed,
    TResult? Function(FFIStatusResponseError field0)? error,
  }) {
    return error?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(FFIStatusResponsePending field0)? pending,
    TResult Function(FFIStatusResponseCompleted field0)? completed,
    TResult Function(FFIStatusResponseError field0)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(field0);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FFIStatusResponseData_Pending value) pending,
    required TResult Function(FFIStatusResponseData_Completed value) completed,
    required TResult Function(FFIStatusResponseData_Error value) error,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FFIStatusResponseData_Pending value)? pending,
    TResult? Function(FFIStatusResponseData_Completed value)? completed,
    TResult? Function(FFIStatusResponseData_Error value)? error,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FFIStatusResponseData_Pending value)? pending,
    TResult Function(FFIStatusResponseData_Completed value)? completed,
    TResult Function(FFIStatusResponseData_Error value)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }
}

abstract class FFIStatusResponseData_Error extends FFIStatusResponseData {
  const factory FFIStatusResponseData_Error(
      final FFIStatusResponseError field0) = _$FFIStatusResponseData_ErrorImpl;
  const FFIStatusResponseData_Error._() : super._();

  @override
  FFIStatusResponseError get field0;

  /// Create a copy of FFIStatusResponseData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FFIStatusResponseData_ErrorImplCopyWith<_$FFIStatusResponseData_ErrorImpl>
      get copyWith => throw _privateConstructorUsedError;
}
