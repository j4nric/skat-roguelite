import Capacitor
import UIKit

/// The app's web view (ADR 0026): Capacitor's bridge around the bundled
/// web build, plus the app's own plugins: passkeys and the session token
/// (ADR 0026), and the full version through StoreKit (ADR 0028).
class AppViewController: CAPBridgeViewController {
    override func capacitorDidLoad() {
        bridge?.registerPluginInstance(NativeAccountPlugin())
        bridge?.registerPluginInstance(StorePlugin())
    }
}
