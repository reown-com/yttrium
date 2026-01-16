import SwiftUI

#if canImport(Yttrium)
import Yttrium
#endif

struct ContentView: View {
    var body: some View {
        VStack {
            Text("Yttrium Size Test")
                .font(.title)

            #if canImport(Yttrium)
            Text("Yttrium is linked")
                .foregroundColor(.green)
            #else
            Text("Baseline (no Yttrium)")
                .foregroundColor(.gray)
            #endif
        }
        .padding()
    }
}
