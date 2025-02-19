// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'dart_compat_models.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$ErrorCompat {
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
    required TResult Function(ErrorCompat_General value) general,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ErrorCompat_General value)? general,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ErrorCompat_General value)? general,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;

  /// Create a copy of ErrorCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ErrorCompatCopyWith<ErrorCompat> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ErrorCompatCopyWith<$Res> {
  factory $ErrorCompatCopyWith(
          ErrorCompat value, $Res Function(ErrorCompat) then) =
      _$ErrorCompatCopyWithImpl<$Res, ErrorCompat>;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class _$ErrorCompatCopyWithImpl<$Res, $Val extends ErrorCompat>
    implements $ErrorCompatCopyWith<$Res> {
  _$ErrorCompatCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ErrorCompat
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
abstract class _$$ErrorCompat_GeneralImplCopyWith<$Res>
    implements $ErrorCompatCopyWith<$Res> {
  factory _$$ErrorCompat_GeneralImplCopyWith(_$ErrorCompat_GeneralImpl value,
          $Res Function(_$ErrorCompat_GeneralImpl) then) =
      __$$ErrorCompat_GeneralImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String field0});
}

/// @nodoc
class __$$ErrorCompat_GeneralImplCopyWithImpl<$Res>
    extends _$ErrorCompatCopyWithImpl<$Res, _$ErrorCompat_GeneralImpl>
    implements _$$ErrorCompat_GeneralImplCopyWith<$Res> {
  __$$ErrorCompat_GeneralImplCopyWithImpl(_$ErrorCompat_GeneralImpl _value,
      $Res Function(_$ErrorCompat_GeneralImpl) _then)
      : super(_value, _then);

  /// Create a copy of ErrorCompat
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$ErrorCompat_GeneralImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$ErrorCompat_GeneralImpl extends ErrorCompat_General {
  const _$ErrorCompat_GeneralImpl(this.field0) : super._();

  @override
  final String field0;

  @override
  String toString() {
    return 'ErrorCompat.general(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ErrorCompat_GeneralImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of ErrorCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ErrorCompat_GeneralImplCopyWith<_$ErrorCompat_GeneralImpl> get copyWith =>
      __$$ErrorCompat_GeneralImplCopyWithImpl<_$ErrorCompat_GeneralImpl>(
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
    required TResult Function(ErrorCompat_General value) general,
  }) {
    return general(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ErrorCompat_General value)? general,
  }) {
    return general?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ErrorCompat_General value)? general,
    required TResult orElse(),
  }) {
    if (general != null) {
      return general(this);
    }
    return orElse();
  }
}

abstract class ErrorCompat_General extends ErrorCompat {
  const factory ErrorCompat_General(final String field0) =
      _$ErrorCompat_GeneralImpl;
  const ErrorCompat_General._() : super._();

  @override
  String get field0;

  /// Create a copy of ErrorCompat
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ErrorCompat_GeneralImplCopyWith<_$ErrorCompat_GeneralImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$PrepareDetailedResponseCompat {
  Object get value => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PrepareDetailedResponseSuccessCompat value)
        success,
    required TResult Function(PrepareResponseError value) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PrepareDetailedResponseSuccessCompat value)? success,
    TResult? Function(PrepareResponseError value)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PrepareDetailedResponseSuccessCompat value)? success,
    TResult Function(PrepareResponseError value)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PrepareDetailedResponseCompat_Success value)
        success,
    required TResult Function(PrepareDetailedResponseCompat_Error value) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PrepareDetailedResponseCompat_Success value)? success,
    TResult? Function(PrepareDetailedResponseCompat_Error value)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PrepareDetailedResponseCompat_Success value)? success,
    TResult Function(PrepareDetailedResponseCompat_Error value)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PrepareDetailedResponseCompatCopyWith<$Res> {
  factory $PrepareDetailedResponseCompatCopyWith(
          PrepareDetailedResponseCompat value,
          $Res Function(PrepareDetailedResponseCompat) then) =
      _$PrepareDetailedResponseCompatCopyWithImpl<$Res,
          PrepareDetailedResponseCompat>;
}

/// @nodoc
class _$PrepareDetailedResponseCompatCopyWithImpl<$Res,
        $Val extends PrepareDetailedResponseCompat>
    implements $PrepareDetailedResponseCompatCopyWith<$Res> {
  _$PrepareDetailedResponseCompatCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PrepareDetailedResponseCompat
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$PrepareDetailedResponseCompat_SuccessImplCopyWith<$Res> {
  factory _$$PrepareDetailedResponseCompat_SuccessImplCopyWith(
          _$PrepareDetailedResponseCompat_SuccessImpl value,
          $Res Function(_$PrepareDetailedResponseCompat_SuccessImpl) then) =
      __$$PrepareDetailedResponseCompat_SuccessImplCopyWithImpl<$Res>;
  @useResult
  $Res call({PrepareDetailedResponseSuccessCompat value});

  $PrepareDetailedResponseSuccessCompatCopyWith<$Res> get value;
}

/// @nodoc
class __$$PrepareDetailedResponseCompat_SuccessImplCopyWithImpl<$Res>
    extends _$PrepareDetailedResponseCompatCopyWithImpl<$Res,
        _$PrepareDetailedResponseCompat_SuccessImpl>
    implements _$$PrepareDetailedResponseCompat_SuccessImplCopyWith<$Res> {
  __$$PrepareDetailedResponseCompat_SuccessImplCopyWithImpl(
      _$PrepareDetailedResponseCompat_SuccessImpl _value,
      $Res Function(_$PrepareDetailedResponseCompat_SuccessImpl) _then)
      : super(_value, _then);

  /// Create a copy of PrepareDetailedResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$PrepareDetailedResponseCompat_SuccessImpl(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as PrepareDetailedResponseSuccessCompat,
    ));
  }

  /// Create a copy of PrepareDetailedResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $PrepareDetailedResponseSuccessCompatCopyWith<$Res> get value {
    return $PrepareDetailedResponseSuccessCompatCopyWith<$Res>(_value.value,
        (value) {
      return _then(_value.copyWith(value: value));
    });
  }
}

/// @nodoc

class _$PrepareDetailedResponseCompat_SuccessImpl
    extends PrepareDetailedResponseCompat_Success {
  const _$PrepareDetailedResponseCompat_SuccessImpl({required this.value})
      : super._();

  @override
  final PrepareDetailedResponseSuccessCompat value;

  @override
  String toString() {
    return 'PrepareDetailedResponseCompat.success(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PrepareDetailedResponseCompat_SuccessImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of PrepareDetailedResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PrepareDetailedResponseCompat_SuccessImplCopyWith<
          _$PrepareDetailedResponseCompat_SuccessImpl>
      get copyWith => __$$PrepareDetailedResponseCompat_SuccessImplCopyWithImpl<
          _$PrepareDetailedResponseCompat_SuccessImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PrepareDetailedResponseSuccessCompat value)
        success,
    required TResult Function(PrepareResponseError value) error,
  }) {
    return success(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PrepareDetailedResponseSuccessCompat value)? success,
    TResult? Function(PrepareResponseError value)? error,
  }) {
    return success?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PrepareDetailedResponseSuccessCompat value)? success,
    TResult Function(PrepareResponseError value)? error,
    required TResult orElse(),
  }) {
    if (success != null) {
      return success(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PrepareDetailedResponseCompat_Success value)
        success,
    required TResult Function(PrepareDetailedResponseCompat_Error value) error,
  }) {
    return success(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PrepareDetailedResponseCompat_Success value)? success,
    TResult? Function(PrepareDetailedResponseCompat_Error value)? error,
  }) {
    return success?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PrepareDetailedResponseCompat_Success value)? success,
    TResult Function(PrepareDetailedResponseCompat_Error value)? error,
    required TResult orElse(),
  }) {
    if (success != null) {
      return success(this);
    }
    return orElse();
  }
}

abstract class PrepareDetailedResponseCompat_Success
    extends PrepareDetailedResponseCompat {
  const factory PrepareDetailedResponseCompat_Success(
          {required final PrepareDetailedResponseSuccessCompat value}) =
      _$PrepareDetailedResponseCompat_SuccessImpl;
  const PrepareDetailedResponseCompat_Success._() : super._();

  @override
  PrepareDetailedResponseSuccessCompat get value;

  /// Create a copy of PrepareDetailedResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PrepareDetailedResponseCompat_SuccessImplCopyWith<
          _$PrepareDetailedResponseCompat_SuccessImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PrepareDetailedResponseCompat_ErrorImplCopyWith<$Res> {
  factory _$$PrepareDetailedResponseCompat_ErrorImplCopyWith(
          _$PrepareDetailedResponseCompat_ErrorImpl value,
          $Res Function(_$PrepareDetailedResponseCompat_ErrorImpl) then) =
      __$$PrepareDetailedResponseCompat_ErrorImplCopyWithImpl<$Res>;
  @useResult
  $Res call({PrepareResponseError value});
}

/// @nodoc
class __$$PrepareDetailedResponseCompat_ErrorImplCopyWithImpl<$Res>
    extends _$PrepareDetailedResponseCompatCopyWithImpl<$Res,
        _$PrepareDetailedResponseCompat_ErrorImpl>
    implements _$$PrepareDetailedResponseCompat_ErrorImplCopyWith<$Res> {
  __$$PrepareDetailedResponseCompat_ErrorImplCopyWithImpl(
      _$PrepareDetailedResponseCompat_ErrorImpl _value,
      $Res Function(_$PrepareDetailedResponseCompat_ErrorImpl) _then)
      : super(_value, _then);

  /// Create a copy of PrepareDetailedResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$PrepareDetailedResponseCompat_ErrorImpl(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as PrepareResponseError,
    ));
  }
}

/// @nodoc

class _$PrepareDetailedResponseCompat_ErrorImpl
    extends PrepareDetailedResponseCompat_Error {
  const _$PrepareDetailedResponseCompat_ErrorImpl({required this.value})
      : super._();

  @override
  final PrepareResponseError value;

  @override
  String toString() {
    return 'PrepareDetailedResponseCompat.error(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PrepareDetailedResponseCompat_ErrorImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of PrepareDetailedResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PrepareDetailedResponseCompat_ErrorImplCopyWith<
          _$PrepareDetailedResponseCompat_ErrorImpl>
      get copyWith => __$$PrepareDetailedResponseCompat_ErrorImplCopyWithImpl<
          _$PrepareDetailedResponseCompat_ErrorImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PrepareDetailedResponseSuccessCompat value)
        success,
    required TResult Function(PrepareResponseError value) error,
  }) {
    return error(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PrepareDetailedResponseSuccessCompat value)? success,
    TResult? Function(PrepareResponseError value)? error,
  }) {
    return error?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PrepareDetailedResponseSuccessCompat value)? success,
    TResult Function(PrepareResponseError value)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PrepareDetailedResponseCompat_Success value)
        success,
    required TResult Function(PrepareDetailedResponseCompat_Error value) error,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PrepareDetailedResponseCompat_Success value)? success,
    TResult? Function(PrepareDetailedResponseCompat_Error value)? error,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PrepareDetailedResponseCompat_Success value)? success,
    TResult Function(PrepareDetailedResponseCompat_Error value)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }
}

abstract class PrepareDetailedResponseCompat_Error
    extends PrepareDetailedResponseCompat {
  const factory PrepareDetailedResponseCompat_Error(
          {required final PrepareResponseError value}) =
      _$PrepareDetailedResponseCompat_ErrorImpl;
  const PrepareDetailedResponseCompat_Error._() : super._();

  @override
  PrepareResponseError get value;

  /// Create a copy of PrepareDetailedResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PrepareDetailedResponseCompat_ErrorImplCopyWith<
          _$PrepareDetailedResponseCompat_ErrorImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$PrepareDetailedResponseSuccessCompat {
  Object get value => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(UiFieldsCompat value) available,
    required TResult Function(PrepareResponseNotRequiredCompat value)
        notRequired,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(UiFieldsCompat value)? available,
    TResult? Function(PrepareResponseNotRequiredCompat value)? notRequired,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(UiFieldsCompat value)? available,
    TResult Function(PrepareResponseNotRequiredCompat value)? notRequired,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(
            PrepareDetailedResponseSuccessCompat_Available value)
        available,
    required TResult Function(
            PrepareDetailedResponseSuccessCompat_NotRequired value)
        notRequired,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PrepareDetailedResponseSuccessCompat_Available value)?
        available,
    TResult? Function(PrepareDetailedResponseSuccessCompat_NotRequired value)?
        notRequired,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PrepareDetailedResponseSuccessCompat_Available value)?
        available,
    TResult Function(PrepareDetailedResponseSuccessCompat_NotRequired value)?
        notRequired,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PrepareDetailedResponseSuccessCompatCopyWith<$Res> {
  factory $PrepareDetailedResponseSuccessCompatCopyWith(
          PrepareDetailedResponseSuccessCompat value,
          $Res Function(PrepareDetailedResponseSuccessCompat) then) =
      _$PrepareDetailedResponseSuccessCompatCopyWithImpl<$Res,
          PrepareDetailedResponseSuccessCompat>;
}

/// @nodoc
class _$PrepareDetailedResponseSuccessCompatCopyWithImpl<$Res,
        $Val extends PrepareDetailedResponseSuccessCompat>
    implements $PrepareDetailedResponseSuccessCompatCopyWith<$Res> {
  _$PrepareDetailedResponseSuccessCompatCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PrepareDetailedResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$PrepareDetailedResponseSuccessCompat_AvailableImplCopyWith<
    $Res> {
  factory _$$PrepareDetailedResponseSuccessCompat_AvailableImplCopyWith(
          _$PrepareDetailedResponseSuccessCompat_AvailableImpl value,
          $Res Function(_$PrepareDetailedResponseSuccessCompat_AvailableImpl)
              then) =
      __$$PrepareDetailedResponseSuccessCompat_AvailableImplCopyWithImpl<$Res>;
  @useResult
  $Res call({UiFieldsCompat value});
}

/// @nodoc
class __$$PrepareDetailedResponseSuccessCompat_AvailableImplCopyWithImpl<$Res>
    extends _$PrepareDetailedResponseSuccessCompatCopyWithImpl<$Res,
        _$PrepareDetailedResponseSuccessCompat_AvailableImpl>
    implements
        _$$PrepareDetailedResponseSuccessCompat_AvailableImplCopyWith<$Res> {
  __$$PrepareDetailedResponseSuccessCompat_AvailableImplCopyWithImpl(
      _$PrepareDetailedResponseSuccessCompat_AvailableImpl _value,
      $Res Function(_$PrepareDetailedResponseSuccessCompat_AvailableImpl) _then)
      : super(_value, _then);

  /// Create a copy of PrepareDetailedResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$PrepareDetailedResponseSuccessCompat_AvailableImpl(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as UiFieldsCompat,
    ));
  }
}

/// @nodoc

class _$PrepareDetailedResponseSuccessCompat_AvailableImpl
    extends PrepareDetailedResponseSuccessCompat_Available {
  const _$PrepareDetailedResponseSuccessCompat_AvailableImpl(
      {required this.value})
      : super._();

  @override
  final UiFieldsCompat value;

  @override
  String toString() {
    return 'PrepareDetailedResponseSuccessCompat.available(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PrepareDetailedResponseSuccessCompat_AvailableImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of PrepareDetailedResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PrepareDetailedResponseSuccessCompat_AvailableImplCopyWith<
          _$PrepareDetailedResponseSuccessCompat_AvailableImpl>
      get copyWith =>
          __$$PrepareDetailedResponseSuccessCompat_AvailableImplCopyWithImpl<
                  _$PrepareDetailedResponseSuccessCompat_AvailableImpl>(
              this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(UiFieldsCompat value) available,
    required TResult Function(PrepareResponseNotRequiredCompat value)
        notRequired,
  }) {
    return available(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(UiFieldsCompat value)? available,
    TResult? Function(PrepareResponseNotRequiredCompat value)? notRequired,
  }) {
    return available?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(UiFieldsCompat value)? available,
    TResult Function(PrepareResponseNotRequiredCompat value)? notRequired,
    required TResult orElse(),
  }) {
    if (available != null) {
      return available(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(
            PrepareDetailedResponseSuccessCompat_Available value)
        available,
    required TResult Function(
            PrepareDetailedResponseSuccessCompat_NotRequired value)
        notRequired,
  }) {
    return available(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PrepareDetailedResponseSuccessCompat_Available value)?
        available,
    TResult? Function(PrepareDetailedResponseSuccessCompat_NotRequired value)?
        notRequired,
  }) {
    return available?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PrepareDetailedResponseSuccessCompat_Available value)?
        available,
    TResult Function(PrepareDetailedResponseSuccessCompat_NotRequired value)?
        notRequired,
    required TResult orElse(),
  }) {
    if (available != null) {
      return available(this);
    }
    return orElse();
  }
}

abstract class PrepareDetailedResponseSuccessCompat_Available
    extends PrepareDetailedResponseSuccessCompat {
  const factory PrepareDetailedResponseSuccessCompat_Available(
          {required final UiFieldsCompat value}) =
      _$PrepareDetailedResponseSuccessCompat_AvailableImpl;
  const PrepareDetailedResponseSuccessCompat_Available._() : super._();

  @override
  UiFieldsCompat get value;

  /// Create a copy of PrepareDetailedResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PrepareDetailedResponseSuccessCompat_AvailableImplCopyWith<
          _$PrepareDetailedResponseSuccessCompat_AvailableImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PrepareDetailedResponseSuccessCompat_NotRequiredImplCopyWith<
    $Res> {
  factory _$$PrepareDetailedResponseSuccessCompat_NotRequiredImplCopyWith(
          _$PrepareDetailedResponseSuccessCompat_NotRequiredImpl value,
          $Res Function(_$PrepareDetailedResponseSuccessCompat_NotRequiredImpl)
              then) =
      __$$PrepareDetailedResponseSuccessCompat_NotRequiredImplCopyWithImpl<
          $Res>;
  @useResult
  $Res call({PrepareResponseNotRequiredCompat value});
}

/// @nodoc
class __$$PrepareDetailedResponseSuccessCompat_NotRequiredImplCopyWithImpl<$Res>
    extends _$PrepareDetailedResponseSuccessCompatCopyWithImpl<$Res,
        _$PrepareDetailedResponseSuccessCompat_NotRequiredImpl>
    implements
        _$$PrepareDetailedResponseSuccessCompat_NotRequiredImplCopyWith<$Res> {
  __$$PrepareDetailedResponseSuccessCompat_NotRequiredImplCopyWithImpl(
      _$PrepareDetailedResponseSuccessCompat_NotRequiredImpl _value,
      $Res Function(_$PrepareDetailedResponseSuccessCompat_NotRequiredImpl)
          _then)
      : super(_value, _then);

  /// Create a copy of PrepareDetailedResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$PrepareDetailedResponseSuccessCompat_NotRequiredImpl(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as PrepareResponseNotRequiredCompat,
    ));
  }
}

/// @nodoc

class _$PrepareDetailedResponseSuccessCompat_NotRequiredImpl
    extends PrepareDetailedResponseSuccessCompat_NotRequired {
  const _$PrepareDetailedResponseSuccessCompat_NotRequiredImpl(
      {required this.value})
      : super._();

  @override
  final PrepareResponseNotRequiredCompat value;

  @override
  String toString() {
    return 'PrepareDetailedResponseSuccessCompat.notRequired(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PrepareDetailedResponseSuccessCompat_NotRequiredImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of PrepareDetailedResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PrepareDetailedResponseSuccessCompat_NotRequiredImplCopyWith<
          _$PrepareDetailedResponseSuccessCompat_NotRequiredImpl>
      get copyWith =>
          __$$PrepareDetailedResponseSuccessCompat_NotRequiredImplCopyWithImpl<
                  _$PrepareDetailedResponseSuccessCompat_NotRequiredImpl>(
              this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(UiFieldsCompat value) available,
    required TResult Function(PrepareResponseNotRequiredCompat value)
        notRequired,
  }) {
    return notRequired(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(UiFieldsCompat value)? available,
    TResult? Function(PrepareResponseNotRequiredCompat value)? notRequired,
  }) {
    return notRequired?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(UiFieldsCompat value)? available,
    TResult Function(PrepareResponseNotRequiredCompat value)? notRequired,
    required TResult orElse(),
  }) {
    if (notRequired != null) {
      return notRequired(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(
            PrepareDetailedResponseSuccessCompat_Available value)
        available,
    required TResult Function(
            PrepareDetailedResponseSuccessCompat_NotRequired value)
        notRequired,
  }) {
    return notRequired(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PrepareDetailedResponseSuccessCompat_Available value)?
        available,
    TResult? Function(PrepareDetailedResponseSuccessCompat_NotRequired value)?
        notRequired,
  }) {
    return notRequired?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PrepareDetailedResponseSuccessCompat_Available value)?
        available,
    TResult Function(PrepareDetailedResponseSuccessCompat_NotRequired value)?
        notRequired,
    required TResult orElse(),
  }) {
    if (notRequired != null) {
      return notRequired(this);
    }
    return orElse();
  }
}

abstract class PrepareDetailedResponseSuccessCompat_NotRequired
    extends PrepareDetailedResponseSuccessCompat {
  const factory PrepareDetailedResponseSuccessCompat_NotRequired(
          {required final PrepareResponseNotRequiredCompat value}) =
      _$PrepareDetailedResponseSuccessCompat_NotRequiredImpl;
  const PrepareDetailedResponseSuccessCompat_NotRequired._() : super._();

  @override
  PrepareResponseNotRequiredCompat get value;

  /// Create a copy of PrepareDetailedResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PrepareDetailedResponseSuccessCompat_NotRequiredImplCopyWith<
          _$PrepareDetailedResponseSuccessCompat_NotRequiredImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$PrepareResponseCompat {
  Object get field0 => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PrepareResponseSuccessCompat field0) success,
    required TResult Function(PrepareResponseErrorCompat field0) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PrepareResponseSuccessCompat field0)? success,
    TResult? Function(PrepareResponseErrorCompat field0)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PrepareResponseSuccessCompat field0)? success,
    TResult Function(PrepareResponseErrorCompat field0)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PrepareResponseCompat_Success value) success,
    required TResult Function(PrepareResponseCompat_Error value) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PrepareResponseCompat_Success value)? success,
    TResult? Function(PrepareResponseCompat_Error value)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PrepareResponseCompat_Success value)? success,
    TResult Function(PrepareResponseCompat_Error value)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PrepareResponseCompatCopyWith<$Res> {
  factory $PrepareResponseCompatCopyWith(PrepareResponseCompat value,
          $Res Function(PrepareResponseCompat) then) =
      _$PrepareResponseCompatCopyWithImpl<$Res, PrepareResponseCompat>;
}

/// @nodoc
class _$PrepareResponseCompatCopyWithImpl<$Res,
        $Val extends PrepareResponseCompat>
    implements $PrepareResponseCompatCopyWith<$Res> {
  _$PrepareResponseCompatCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PrepareResponseCompat
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$PrepareResponseCompat_SuccessImplCopyWith<$Res> {
  factory _$$PrepareResponseCompat_SuccessImplCopyWith(
          _$PrepareResponseCompat_SuccessImpl value,
          $Res Function(_$PrepareResponseCompat_SuccessImpl) then) =
      __$$PrepareResponseCompat_SuccessImplCopyWithImpl<$Res>;
  @useResult
  $Res call({PrepareResponseSuccessCompat field0});

  $PrepareResponseSuccessCompatCopyWith<$Res> get field0;
}

/// @nodoc
class __$$PrepareResponseCompat_SuccessImplCopyWithImpl<$Res>
    extends _$PrepareResponseCompatCopyWithImpl<$Res,
        _$PrepareResponseCompat_SuccessImpl>
    implements _$$PrepareResponseCompat_SuccessImplCopyWith<$Res> {
  __$$PrepareResponseCompat_SuccessImplCopyWithImpl(
      _$PrepareResponseCompat_SuccessImpl _value,
      $Res Function(_$PrepareResponseCompat_SuccessImpl) _then)
      : super(_value, _then);

  /// Create a copy of PrepareResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$PrepareResponseCompat_SuccessImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as PrepareResponseSuccessCompat,
    ));
  }

  /// Create a copy of PrepareResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $PrepareResponseSuccessCompatCopyWith<$Res> get field0 {
    return $PrepareResponseSuccessCompatCopyWith<$Res>(_value.field0, (value) {
      return _then(_value.copyWith(field0: value));
    });
  }
}

/// @nodoc

class _$PrepareResponseCompat_SuccessImpl
    extends PrepareResponseCompat_Success {
  const _$PrepareResponseCompat_SuccessImpl(this.field0) : super._();

  @override
  final PrepareResponseSuccessCompat field0;

  @override
  String toString() {
    return 'PrepareResponseCompat.success(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PrepareResponseCompat_SuccessImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of PrepareResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PrepareResponseCompat_SuccessImplCopyWith<
          _$PrepareResponseCompat_SuccessImpl>
      get copyWith => __$$PrepareResponseCompat_SuccessImplCopyWithImpl<
          _$PrepareResponseCompat_SuccessImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PrepareResponseSuccessCompat field0) success,
    required TResult Function(PrepareResponseErrorCompat field0) error,
  }) {
    return success(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PrepareResponseSuccessCompat field0)? success,
    TResult? Function(PrepareResponseErrorCompat field0)? error,
  }) {
    return success?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PrepareResponseSuccessCompat field0)? success,
    TResult Function(PrepareResponseErrorCompat field0)? error,
    required TResult orElse(),
  }) {
    if (success != null) {
      return success(field0);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PrepareResponseCompat_Success value) success,
    required TResult Function(PrepareResponseCompat_Error value) error,
  }) {
    return success(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PrepareResponseCompat_Success value)? success,
    TResult? Function(PrepareResponseCompat_Error value)? error,
  }) {
    return success?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PrepareResponseCompat_Success value)? success,
    TResult Function(PrepareResponseCompat_Error value)? error,
    required TResult orElse(),
  }) {
    if (success != null) {
      return success(this);
    }
    return orElse();
  }
}

abstract class PrepareResponseCompat_Success extends PrepareResponseCompat {
  const factory PrepareResponseCompat_Success(
          final PrepareResponseSuccessCompat field0) =
      _$PrepareResponseCompat_SuccessImpl;
  const PrepareResponseCompat_Success._() : super._();

  @override
  PrepareResponseSuccessCompat get field0;

  /// Create a copy of PrepareResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PrepareResponseCompat_SuccessImplCopyWith<
          _$PrepareResponseCompat_SuccessImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PrepareResponseCompat_ErrorImplCopyWith<$Res> {
  factory _$$PrepareResponseCompat_ErrorImplCopyWith(
          _$PrepareResponseCompat_ErrorImpl value,
          $Res Function(_$PrepareResponseCompat_ErrorImpl) then) =
      __$$PrepareResponseCompat_ErrorImplCopyWithImpl<$Res>;
  @useResult
  $Res call({PrepareResponseErrorCompat field0});
}

/// @nodoc
class __$$PrepareResponseCompat_ErrorImplCopyWithImpl<$Res>
    extends _$PrepareResponseCompatCopyWithImpl<$Res,
        _$PrepareResponseCompat_ErrorImpl>
    implements _$$PrepareResponseCompat_ErrorImplCopyWith<$Res> {
  __$$PrepareResponseCompat_ErrorImplCopyWithImpl(
      _$PrepareResponseCompat_ErrorImpl _value,
      $Res Function(_$PrepareResponseCompat_ErrorImpl) _then)
      : super(_value, _then);

  /// Create a copy of PrepareResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$PrepareResponseCompat_ErrorImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as PrepareResponseErrorCompat,
    ));
  }
}

/// @nodoc

class _$PrepareResponseCompat_ErrorImpl extends PrepareResponseCompat_Error {
  const _$PrepareResponseCompat_ErrorImpl(this.field0) : super._();

  @override
  final PrepareResponseErrorCompat field0;

  @override
  String toString() {
    return 'PrepareResponseCompat.error(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PrepareResponseCompat_ErrorImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of PrepareResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PrepareResponseCompat_ErrorImplCopyWith<_$PrepareResponseCompat_ErrorImpl>
      get copyWith => __$$PrepareResponseCompat_ErrorImplCopyWithImpl<
          _$PrepareResponseCompat_ErrorImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PrepareResponseSuccessCompat field0) success,
    required TResult Function(PrepareResponseErrorCompat field0) error,
  }) {
    return error(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PrepareResponseSuccessCompat field0)? success,
    TResult? Function(PrepareResponseErrorCompat field0)? error,
  }) {
    return error?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PrepareResponseSuccessCompat field0)? success,
    TResult Function(PrepareResponseErrorCompat field0)? error,
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
    required TResult Function(PrepareResponseCompat_Success value) success,
    required TResult Function(PrepareResponseCompat_Error value) error,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PrepareResponseCompat_Success value)? success,
    TResult? Function(PrepareResponseCompat_Error value)? error,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PrepareResponseCompat_Success value)? success,
    TResult Function(PrepareResponseCompat_Error value)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }
}

abstract class PrepareResponseCompat_Error extends PrepareResponseCompat {
  const factory PrepareResponseCompat_Error(
          final PrepareResponseErrorCompat field0) =
      _$PrepareResponseCompat_ErrorImpl;
  const PrepareResponseCompat_Error._() : super._();

  @override
  PrepareResponseErrorCompat get field0;

  /// Create a copy of PrepareResponseCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PrepareResponseCompat_ErrorImplCopyWith<_$PrepareResponseCompat_ErrorImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$PrepareResponseSuccessCompat {
  Object get value => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PrepareResponseAvailableCompat value) available,
    required TResult Function(PrepareResponseNotRequiredCompat value)
        notRequired,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PrepareResponseAvailableCompat value)? available,
    TResult? Function(PrepareResponseNotRequiredCompat value)? notRequired,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PrepareResponseAvailableCompat value)? available,
    TResult Function(PrepareResponseNotRequiredCompat value)? notRequired,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PrepareResponseSuccessCompat_Available value)
        available,
    required TResult Function(PrepareResponseSuccessCompat_NotRequired value)
        notRequired,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PrepareResponseSuccessCompat_Available value)? available,
    TResult? Function(PrepareResponseSuccessCompat_NotRequired value)?
        notRequired,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PrepareResponseSuccessCompat_Available value)? available,
    TResult Function(PrepareResponseSuccessCompat_NotRequired value)?
        notRequired,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PrepareResponseSuccessCompatCopyWith<$Res> {
  factory $PrepareResponseSuccessCompatCopyWith(
          PrepareResponseSuccessCompat value,
          $Res Function(PrepareResponseSuccessCompat) then) =
      _$PrepareResponseSuccessCompatCopyWithImpl<$Res,
          PrepareResponseSuccessCompat>;
}

/// @nodoc
class _$PrepareResponseSuccessCompatCopyWithImpl<$Res,
        $Val extends PrepareResponseSuccessCompat>
    implements $PrepareResponseSuccessCompatCopyWith<$Res> {
  _$PrepareResponseSuccessCompatCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PrepareResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$PrepareResponseSuccessCompat_AvailableImplCopyWith<$Res> {
  factory _$$PrepareResponseSuccessCompat_AvailableImplCopyWith(
          _$PrepareResponseSuccessCompat_AvailableImpl value,
          $Res Function(_$PrepareResponseSuccessCompat_AvailableImpl) then) =
      __$$PrepareResponseSuccessCompat_AvailableImplCopyWithImpl<$Res>;
  @useResult
  $Res call({PrepareResponseAvailableCompat value});
}

/// @nodoc
class __$$PrepareResponseSuccessCompat_AvailableImplCopyWithImpl<$Res>
    extends _$PrepareResponseSuccessCompatCopyWithImpl<$Res,
        _$PrepareResponseSuccessCompat_AvailableImpl>
    implements _$$PrepareResponseSuccessCompat_AvailableImplCopyWith<$Res> {
  __$$PrepareResponseSuccessCompat_AvailableImplCopyWithImpl(
      _$PrepareResponseSuccessCompat_AvailableImpl _value,
      $Res Function(_$PrepareResponseSuccessCompat_AvailableImpl) _then)
      : super(_value, _then);

  /// Create a copy of PrepareResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$PrepareResponseSuccessCompat_AvailableImpl(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as PrepareResponseAvailableCompat,
    ));
  }
}

/// @nodoc

class _$PrepareResponseSuccessCompat_AvailableImpl
    extends PrepareResponseSuccessCompat_Available {
  const _$PrepareResponseSuccessCompat_AvailableImpl({required this.value})
      : super._();

  @override
  final PrepareResponseAvailableCompat value;

  @override
  String toString() {
    return 'PrepareResponseSuccessCompat.available(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PrepareResponseSuccessCompat_AvailableImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of PrepareResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PrepareResponseSuccessCompat_AvailableImplCopyWith<
          _$PrepareResponseSuccessCompat_AvailableImpl>
      get copyWith =>
          __$$PrepareResponseSuccessCompat_AvailableImplCopyWithImpl<
              _$PrepareResponseSuccessCompat_AvailableImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PrepareResponseAvailableCompat value) available,
    required TResult Function(PrepareResponseNotRequiredCompat value)
        notRequired,
  }) {
    return available(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PrepareResponseAvailableCompat value)? available,
    TResult? Function(PrepareResponseNotRequiredCompat value)? notRequired,
  }) {
    return available?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PrepareResponseAvailableCompat value)? available,
    TResult Function(PrepareResponseNotRequiredCompat value)? notRequired,
    required TResult orElse(),
  }) {
    if (available != null) {
      return available(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PrepareResponseSuccessCompat_Available value)
        available,
    required TResult Function(PrepareResponseSuccessCompat_NotRequired value)
        notRequired,
  }) {
    return available(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PrepareResponseSuccessCompat_Available value)? available,
    TResult? Function(PrepareResponseSuccessCompat_NotRequired value)?
        notRequired,
  }) {
    return available?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PrepareResponseSuccessCompat_Available value)? available,
    TResult Function(PrepareResponseSuccessCompat_NotRequired value)?
        notRequired,
    required TResult orElse(),
  }) {
    if (available != null) {
      return available(this);
    }
    return orElse();
  }
}

abstract class PrepareResponseSuccessCompat_Available
    extends PrepareResponseSuccessCompat {
  const factory PrepareResponseSuccessCompat_Available(
          {required final PrepareResponseAvailableCompat value}) =
      _$PrepareResponseSuccessCompat_AvailableImpl;
  const PrepareResponseSuccessCompat_Available._() : super._();

  @override
  PrepareResponseAvailableCompat get value;

  /// Create a copy of PrepareResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PrepareResponseSuccessCompat_AvailableImplCopyWith<
          _$PrepareResponseSuccessCompat_AvailableImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PrepareResponseSuccessCompat_NotRequiredImplCopyWith<$Res> {
  factory _$$PrepareResponseSuccessCompat_NotRequiredImplCopyWith(
          _$PrepareResponseSuccessCompat_NotRequiredImpl value,
          $Res Function(_$PrepareResponseSuccessCompat_NotRequiredImpl) then) =
      __$$PrepareResponseSuccessCompat_NotRequiredImplCopyWithImpl<$Res>;
  @useResult
  $Res call({PrepareResponseNotRequiredCompat value});
}

/// @nodoc
class __$$PrepareResponseSuccessCompat_NotRequiredImplCopyWithImpl<$Res>
    extends _$PrepareResponseSuccessCompatCopyWithImpl<$Res,
        _$PrepareResponseSuccessCompat_NotRequiredImpl>
    implements _$$PrepareResponseSuccessCompat_NotRequiredImplCopyWith<$Res> {
  __$$PrepareResponseSuccessCompat_NotRequiredImplCopyWithImpl(
      _$PrepareResponseSuccessCompat_NotRequiredImpl _value,
      $Res Function(_$PrepareResponseSuccessCompat_NotRequiredImpl) _then)
      : super(_value, _then);

  /// Create a copy of PrepareResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? value = null,
  }) {
    return _then(_$PrepareResponseSuccessCompat_NotRequiredImpl(
      value: null == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as PrepareResponseNotRequiredCompat,
    ));
  }
}

/// @nodoc

class _$PrepareResponseSuccessCompat_NotRequiredImpl
    extends PrepareResponseSuccessCompat_NotRequired {
  const _$PrepareResponseSuccessCompat_NotRequiredImpl({required this.value})
      : super._();

  @override
  final PrepareResponseNotRequiredCompat value;

  @override
  String toString() {
    return 'PrepareResponseSuccessCompat.notRequired(value: $value)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PrepareResponseSuccessCompat_NotRequiredImpl &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, value);

  /// Create a copy of PrepareResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PrepareResponseSuccessCompat_NotRequiredImplCopyWith<
          _$PrepareResponseSuccessCompat_NotRequiredImpl>
      get copyWith =>
          __$$PrepareResponseSuccessCompat_NotRequiredImplCopyWithImpl<
              _$PrepareResponseSuccessCompat_NotRequiredImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(PrepareResponseAvailableCompat value) available,
    required TResult Function(PrepareResponseNotRequiredCompat value)
        notRequired,
  }) {
    return notRequired(value);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(PrepareResponseAvailableCompat value)? available,
    TResult? Function(PrepareResponseNotRequiredCompat value)? notRequired,
  }) {
    return notRequired?.call(value);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(PrepareResponseAvailableCompat value)? available,
    TResult Function(PrepareResponseNotRequiredCompat value)? notRequired,
    required TResult orElse(),
  }) {
    if (notRequired != null) {
      return notRequired(value);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PrepareResponseSuccessCompat_Available value)
        available,
    required TResult Function(PrepareResponseSuccessCompat_NotRequired value)
        notRequired,
  }) {
    return notRequired(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PrepareResponseSuccessCompat_Available value)? available,
    TResult? Function(PrepareResponseSuccessCompat_NotRequired value)?
        notRequired,
  }) {
    return notRequired?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PrepareResponseSuccessCompat_Available value)? available,
    TResult Function(PrepareResponseSuccessCompat_NotRequired value)?
        notRequired,
    required TResult orElse(),
  }) {
    if (notRequired != null) {
      return notRequired(this);
    }
    return orElse();
  }
}

abstract class PrepareResponseSuccessCompat_NotRequired
    extends PrepareResponseSuccessCompat {
  const factory PrepareResponseSuccessCompat_NotRequired(
          {required final PrepareResponseNotRequiredCompat value}) =
      _$PrepareResponseSuccessCompat_NotRequiredImpl;
  const PrepareResponseSuccessCompat_NotRequired._() : super._();

  @override
  PrepareResponseNotRequiredCompat get value;

  /// Create a copy of PrepareResponseSuccessCompat
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PrepareResponseSuccessCompat_NotRequiredImplCopyWith<
          _$PrepareResponseSuccessCompat_NotRequiredImpl>
      get copyWith => throw _privateConstructorUsedError;
}
