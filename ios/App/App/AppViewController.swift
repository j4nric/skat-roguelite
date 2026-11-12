import Capacitor
import UIKit

/// The app's web view (ADR 0026): Capacitor's bridge around the bundled
/// web build, plus the app's own plugin for passkeys and the session token.
class AppViewController: CAPBridgeViewController {
    override func capacitorDidLoad() {
        bridge?.registerPluginInstance(NativeAccountPlugin())
    }
}
