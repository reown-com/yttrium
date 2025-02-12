// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'status.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$StatusResponse {
  Object get field0 => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(StatusResponsePending field0) pending,
    required TResult Function(StatusResponseCompleted field0) completed,
    required TResult Function(StatusResponseError field0) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(StatusResponsePending field0)? pending,
    TResult? Function(StatusResponseCompleted field0)? completed,
    TResult? Function(StatusResponseError field0)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(StatusResponsePending field0)? pending,
    TResult Function(StatusResponseCompleted field0)? completed,
    TResult Function(StatusResponseError field0)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(StatusResponse_Pending value) pending,
    required TResult Function(StatusResponse_Completed value) completed,
    required TResult Function(StatusResponse_Error value) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(StatusResponse_Pending value)? pending,
    TResult? Function(StatusResponse_Completed value)? completed,
    TResult? Function(StatusResponse_Error value)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(StatusResponse_Pending value)? pending,
    TResult Function(StatusResponse_Completed value)? completed,
    TResult Function(StatusResponse_Error value)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $StatusResponseCopyWith<$Res> {
  factory $StatusResponseCopyWith(
          StatusResponse value, $Res Function(StatusResponse) then) =
      _$StatusResponseCopyWithImpl<$Res, StatusResponse>;
}

/// @nodoc
class _$StatusResponseCopyWithImpl<$Res, $Val extends StatusResponse>
    implements $StatusResponseCopyWith<$Res> {
  _$StatusResponseCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of StatusResponse
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$StatusResponse_PendingImplCopyWith<$Res> {
  factory _$$StatusResponse_PendingImplCopyWith(
          _$StatusResponse_PendingImpl value,
          $Res Function(_$StatusResponse_PendingImpl) then) =
      __$$StatusResponse_PendingImplCopyWithImpl<$Res>;
  @useResult
  $Res call({StatusResponsePending field0});
}

/// @nodoc
class __$$StatusResponse_PendingImplCopyWithImpl<$Res>
    extends _$StatusResponseCopyWithImpl<$Res, _$StatusResponse_PendingImpl>
    implements _$$StatusResponse_PendingImplCopyWith<$Res> {
  __$$StatusResponse_PendingImplCopyWithImpl(
      _$StatusResponse_PendingImpl _value,
      $Res Function(_$StatusResponse_PendingImpl) _then)
      : super(_value, _then);

  /// Create a copy of StatusResponse
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$StatusResponse_PendingImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as StatusResponsePending,
    ));
  }
}

/// @nodoc

class _$StatusResponse_PendingImpl extends StatusResponse_Pending {
  const _$StatusResponse_PendingImpl(this.field0) : super._();

  @override
  final StatusResponsePending field0;

  @override
  String toString() {
    return 'StatusResponse.pending(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$StatusResponse_PendingImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of StatusResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$StatusResponse_PendingImplCopyWith<_$StatusResponse_PendingImpl>
      get copyWith => __$$StatusResponse_PendingImplCopyWithImpl<
          _$StatusResponse_PendingImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(StatusResponsePending field0) pending,
    required TResult Function(StatusResponseCompleted field0) completed,
    required TResult Function(StatusResponseError field0) error,
  }) {
    return pending(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(StatusResponsePending field0)? pending,
    TResult? Function(StatusResponseCompleted field0)? completed,
    TResult? Function(StatusResponseError field0)? error,
  }) {
    return pending?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(StatusResponsePending field0)? pending,
    TResult Function(StatusResponseCompleted field0)? completed,
    TResult Function(StatusResponseError field0)? error,
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
    required TResult Function(StatusResponse_Pending value) pending,
    required TResult Function(StatusResponse_Completed value) completed,
    required TResult Function(StatusResponse_Error value) error,
  }) {
    return pending(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(StatusResponse_Pending value)? pending,
    TResult? Function(StatusResponse_Completed value)? completed,
    TResult? Function(StatusResponse_Error value)? error,
  }) {
    return pending?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(StatusResponse_Pending value)? pending,
    TResult Function(StatusResponse_Completed value)? completed,
    TResult Function(StatusResponse_Error value)? error,
    required TResult orElse(),
  }) {
    if (pending != null) {
      return pending(this);
    }
    return orElse();
  }
}

abstract class StatusResponse_Pending extends StatusResponse {
  const factory StatusResponse_Pending(final StatusResponsePending field0) =
      _$StatusResponse_PendingImpl;
  const StatusResponse_Pending._() : super._();

  @override
  StatusResponsePending get field0;

  /// Create a copy of StatusResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$StatusResponse_PendingImplCopyWith<_$StatusResponse_PendingImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$StatusResponse_CompletedImplCopyWith<$Res> {
  factory _$$StatusResponse_CompletedImplCopyWith(
          _$StatusResponse_CompletedImpl value,
          $Res Function(_$StatusResponse_CompletedImpl) then) =
      __$$StatusResponse_CompletedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({StatusResponseCompleted field0});
}

/// @nodoc
class __$$StatusResponse_CompletedImplCopyWithImpl<$Res>
    extends _$StatusResponseCopyWithImpl<$Res, _$StatusResponse_CompletedImpl>
    implements _$$StatusResponse_CompletedImplCopyWith<$Res> {
  __$$StatusResponse_CompletedImplCopyWithImpl(
      _$StatusResponse_CompletedImpl _value,
      $Res Function(_$StatusResponse_CompletedImpl) _then)
      : super(_value, _then);

  /// Create a copy of StatusResponse
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$StatusResponse_CompletedImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as StatusResponseCompleted,
    ));
  }
}

/// @nodoc

class _$StatusResponse_CompletedImpl extends StatusResponse_Completed {
  const _$StatusResponse_CompletedImpl(this.field0) : super._();

  @override
  final StatusResponseCompleted field0;

  @override
  String toString() {
    return 'StatusResponse.completed(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$StatusResponse_CompletedImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of StatusResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$StatusResponse_CompletedImplCopyWith<_$StatusResponse_CompletedImpl>
      get copyWith => __$$StatusResponse_CompletedImplCopyWithImpl<
          _$StatusResponse_CompletedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(StatusResponsePending field0) pending,
    required TResult Function(StatusResponseCompleted field0) completed,
    required TResult Function(StatusResponseError field0) error,
  }) {
    return completed(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(StatusResponsePending field0)? pending,
    TResult? Function(StatusResponseCompleted field0)? completed,
    TResult? Function(StatusResponseError field0)? error,
  }) {
    return completed?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(StatusResponsePending field0)? pending,
    TResult Function(StatusResponseCompleted field0)? completed,
    TResult Function(StatusResponseError field0)? error,
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
    required TResult Function(StatusResponse_Pending value) pending,
    required TResult Function(StatusResponse_Completed value) completed,
    required TResult Function(StatusResponse_Error value) error,
  }) {
    return completed(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(StatusResponse_Pending value)? pending,
    TResult? Function(StatusResponse_Completed value)? completed,
    TResult? Function(StatusResponse_Error value)? error,
  }) {
    return completed?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(StatusResponse_Pending value)? pending,
    TResult Function(StatusResponse_Completed value)? completed,
    TResult Function(StatusResponse_Error value)? error,
    required TResult orElse(),
  }) {
    if (completed != null) {
      return completed(this);
    }
    return orElse();
  }
}

abstract class StatusResponse_Completed extends StatusResponse {
  const factory StatusResponse_Completed(final StatusResponseCompleted field0) =
      _$StatusResponse_CompletedImpl;
  const StatusResponse_Completed._() : super._();

  @override
  StatusResponseCompleted get field0;

  /// Create a copy of StatusResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$StatusResponse_CompletedImplCopyWith<_$StatusResponse_CompletedImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$StatusResponse_ErrorImplCopyWith<$Res> {
  factory _$$StatusResponse_ErrorImplCopyWith(_$StatusResponse_ErrorImpl value,
          $Res Function(_$StatusResponse_ErrorImpl) then) =
      __$$StatusResponse_ErrorImplCopyWithImpl<$Res>;
  @useResult
  $Res call({StatusResponseError field0});
}

/// @nodoc
class __$$StatusResponse_ErrorImplCopyWithImpl<$Res>
    extends _$StatusResponseCopyWithImpl<$Res, _$StatusResponse_ErrorImpl>
    implements _$$StatusResponse_ErrorImplCopyWith<$Res> {
  __$$StatusResponse_ErrorImplCopyWithImpl(_$StatusResponse_ErrorImpl _value,
      $Res Function(_$StatusResponse_ErrorImpl) _then)
      : super(_value, _then);

  /// Create a copy of StatusResponse
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$StatusResponse_ErrorImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as StatusResponseError,
    ));
  }
}

/// @nodoc

class _$StatusResponse_ErrorImpl extends StatusResponse_Error {
  const _$StatusResponse_ErrorImpl(this.field0) : super._();

  @override
  final StatusResponseError field0;

  @override
  String toString() {
    return 'StatusResponse.error(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$StatusResponse_ErrorImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of StatusResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$StatusResponse_ErrorImplCopyWith<_$StatusResponse_ErrorImpl>
      get copyWith =>
          __$$StatusResponse_ErrorImplCopyWithImpl<_$StatusResponse_ErrorImpl>(
              this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(StatusResponsePending field0) pending,
    required TResult Function(StatusResponseCompleted field0) completed,
    required TResult Function(StatusResponseError field0) error,
  }) {
    return error(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(StatusResponsePending field0)? pending,
    TResult? Function(StatusResponseCompleted field0)? completed,
    TResult? Function(StatusResponseError field0)? error,
  }) {
    return error?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(StatusResponsePending field0)? pending,
    TResult Function(StatusResponseCompleted field0)? completed,
    TResult Function(StatusResponseError field0)? error,
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
    required TResult Function(StatusResponse_Pending value) pending,
    required TResult Function(StatusResponse_Completed value) completed,
    required TResult Function(StatusResponse_Error value) error,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(StatusResponse_Pending value)? pending,
    TResult? Function(StatusResponse_Completed value)? completed,
    TResult? Function(StatusResponse_Error value)? error,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(StatusResponse_Pending value)? pending,
    TResult Function(StatusResponse_Completed value)? completed,
    TResult Function(StatusResponse_Error value)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }
}

abstract class StatusResponse_Error extends StatusResponse {
  const factory StatusResponse_Error(final StatusResponseError field0) =
      _$StatusResponse_ErrorImpl;
  const StatusResponse_Error._() : super._();

  @override
  StatusResponseError get field0;

  /// Create a copy of StatusResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$StatusResponse_ErrorImplCopyWith<_$StatusResponse_ErrorImpl>
      get copyWith => throw _privateConstructorUsedError;
}
