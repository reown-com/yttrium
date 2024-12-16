```dart
import 'package:flutter/material.dart';
import 'dart:async';

import 'package:yttrium_dart/yttrium_dart.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  bool _initiated = false;

  @override
  void initState() {
    super.initState();
    _initYttrium();
  }

  Future<void> _initYttrium() async {
    try {
      const pid = '07429......';
      await YttriumDart.instance.init(projectId: pid);
      setState(() => _initiated = true);
    } catch (e) {
      setState(() => _initiated = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(
          title: const Text('Yttrium example app'),
        ),
        body: Center(
          child: !_initiated
              ? const CircularProgressIndicator()
              : FutureBuilder(
                  future:
                      YttriumDart.instance.estimateFees(chainId: 'eip155:1'),
                  builder: (_, snapshot) {
                    final inGWei =
                        int.parse(snapshot.data?.maxFeePerGas ?? '0') / 1e9;
                    return Text('Gas price: ${inGWei.toStringAsFixed(2)} Gwei');
                  },
                ),
        ),
      ),
    );
  }
}
```