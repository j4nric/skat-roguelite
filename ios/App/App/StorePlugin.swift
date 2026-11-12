import Capacitor
import Foundation
import StoreKit

/// The full version as an In-App Purchase (ADR 0028), the native side of
/// web/src/native-store.ts. StoreKit 2 only, no third-party library and no
/// server of our own: Apple signs every transaction, StoreKit verifies the
/// signature on the device and keeps the entitlements cached, so the check
/// works offline too.
///
/// - `products`: the full version's localized name and price as the App
///   Store has it (never a price in our code).
/// - `purchase`: buys it; answers `purchased`, `cancelled` or `pending`
///   (Ask to Buy, a payment that needs a step first).
/// - `restore`: `AppStore.sync()`, then the entitlement.
/// - `entitlement`: whether a verified, unrevoked transaction for the full
///   version is among `Transaction.currentEntitlements`.
/// - Event `entitlementChanged` (`{ full }`): from `Transaction.updates`,
///   which brings purchases made on another device, Family Sharing, an
///   approved Ask to Buy and refunds (the entitlement goes again).
///
/// Errors reject with a code web/src/store.ts knows: `unavailable` (no
/// product, offline), `cancelled`, `unverified`, `failed`.
/// Nothing here knows the game; the web app decides what the demo shows.
@objc(StorePlugin)
public class StorePlugin: CAPPlugin, CAPBridgedPlugin {
    public let identifier = "StorePlugin"
    public let jsName = "Store"
    public let pluginMethods: [CAPPluginMethod] = [
        CAPPluginMethod(name: "products", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "purchase", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "restore", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "entitlement", returnType: CAPPluginReturnPromise),
    ]

    /// The one product: non-consumable, App Store Connect, docs/ios.md.
    static let fullVersionId = "de.onemoreskat.app.full"

    /// Listens to `Transaction.updates` for as long as the app runs.
    private var updates: Task<Void, Never>?

    override public func load() {
        // Started at once: transactions that arrive while the app was closed
        // (another device, an approved Ask to Buy, a refund) come here first.
        updates = Task.detached(priority: .background) { [weak self] in
            for await result in Transaction.updates {
                if case .verified(let transaction) = result {
                    await transaction.finish()
                }
                let full = await StorePlugin.hasFullVersion()
                self?.notifyListeners("entitlementChanged", data: ["full": full])
            }
        }
    }

    deinit {
        updates?.cancel()
    }

    /// A verified, unrevoked transaction of the full version is current.
    static func hasFullVersion() async -> Bool {
        for await result in Transaction.currentEntitlements {
            guard case .verified(let transaction) = result else { continue }
            if transaction.productID == fullVersionId && transaction.revocationDate == nil {
                return true
            }
        }
        return false
    }

    private static func product() async throws -> Product? {
        try await Product.products(for: [fullVersionId]).first
    }

    @objc func products(_ call: CAPPluginCall) {
        Task {
            do {
                guard let product = try await Self.product() else {
                    call.reject("product not found", "unavailable")
                    return
                }
                call.resolve([
                    "id": product.id,
                    "name": product.displayName,
                    "description": product.description,
                    "price": product.displayPrice,
                ])
            } catch {
                NSLog("Store: products failed: %@", String(describing: error))
                call.reject("products failed", "unavailable")
            }
        }
    }

    @objc func purchase(_ call: CAPPluginCall) {
        Task { @MainActor in
            do {
                guard let product = try await Self.product() else {
                    call.reject("product not found", "unavailable")
                    return
                }
                let result = try await product.purchase()
                switch result {
                case .success(let verification):
                    switch verification {
                    case .verified(let transaction):
                        await transaction.finish()
                        call.resolve(["state": "purchased", "full": true])
                    case .unverified(_, let error):
                        NSLog("Store: unverified purchase: %@", String(describing: error))
                        call.reject("unverified", "unverified")
                    }
                case .userCancelled:
                    call.resolve(["state": "cancelled", "full": await Self.hasFullVersion()])
                case .pending:
                    call.resolve(["state": "pending", "full": false])
                @unknown default:
                    call.reject("unknown purchase result", "failed")
                }
            } catch {
                NSLog("Store: purchase failed: %@", String(describing: error))
                if let storeError = error as? StoreKitError, case .userCancelled = storeError {
                    call.resolve(["state": "cancelled", "full": await Self.hasFullVersion()])
                } else if let storeError = error as? StoreKitError, case .networkError = storeError {
                    call.reject("network", "unavailable")
                } else {
                    call.reject("purchase failed", "failed")
                }
            }
        }
    }

    @objc func restore(_ call: CAPPluginCall) {
        Task { @MainActor in
            do {
                try await AppStore.sync()
            } catch {
                NSLog("Store: restore failed: %@", String(describing: error))
                if let storeError = error as? StoreKitError, case .userCancelled = storeError {
                    call.reject("cancelled", "cancelled")
                    return
                }
                // Offline: the cached entitlements still answer below.
                if await Self.hasFullVersion() {
                    call.resolve(["full": true])
                } else {
                    call.reject("restore failed", "failed")
                }
                return
            }
            call.resolve(["full": await Self.hasFullVersion()])
        }
    }

    @objc func entitlement(_ call: CAPPluginCall) {
        Task {
            call.resolve(["full": await Self.hasFullVersion()])
        }
    }
}
