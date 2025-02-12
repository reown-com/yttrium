// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'dart_compat.dart';

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
