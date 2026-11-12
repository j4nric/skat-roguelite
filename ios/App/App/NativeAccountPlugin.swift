import AuthenticationServices
import Capacitor
import Foundation
import Security
import UIKit
import WebKit

/// The app's own small plugin (ADR 0026), the native side of
/// web/src/native.ts. Transport only, no game logic:
///
/// - `create` and `get`: passkeys through iOS for the relying party the
///   account server names (one-more-skat.de). The web view runs at
///   `capacitor://localhost`, where WebAuthn cannot use that RP id; iOS can,
///   because the app lists `webcredentials:one-more-skat.de` in its
///   Associated Domains and the site names the app in
///   /.well-known/apple-app-site-association. The options come exactly as
///   the server sends them (JSON with base64url strings), and the answer
///   is the JSON the server reads, as web/src/account.ts makes it of a
///   browser credential. iOS writes `https://one-more-skat.de` as the
///   origin into clientDataJSON.
/// - `readSession` and `writeSession`: the session token in the Keychain
///   (this device only), where the web app keeps it instead of a cookie.
///
/// Errors reject with a code web/src/account.ts knows: `cancelled`,
/// `passkey_exists`, `unsupported`, `passkey`.
@objc(NativeAccountPlugin)
public class NativeAccountPlugin: CAPPlugin, CAPBridgedPlugin, ASAuthorizationControllerDelegate,
    ASAuthorizationControllerPresentationContextProviding
{
    public let identifier = "NativeAccountPlugin"
    public let jsName = "NativeAccount"
    public let pluginMethods: [CAPPluginMethod] = [
        CAPPluginMethod(name: "create", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "get", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "readSession", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "writeSession", returnType: CAPPluginReturnPromise),
    ]

    /// The call waiting for the passkey sheet (one at a time).
    private var pending: CAPPluginCall?
    /// Held while the sheet is open.
    private var controller: ASAuthorizationController?

    // MARK: - Passkeys

    @objc func create(_ call: CAPPluginCall) {
        guard let publicKey = call.getObject("publicKey"),
            let rp = publicKey["rp"] as? JSObject,
            let rpId = rp["id"] as? String,
            let user = publicKey["user"] as? JSObject,
            let name = user["name"] as? String,
            let userId = Self.decode(user["id"] as? String),
            let challenge = Self.decode(publicKey["challenge"] as? String)
        else {
            call.reject("bad passkey options", "passkey")
            return
        }
        let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(relyingPartyIdentifier: rpId)
        let request = provider.createCredentialRegistrationRequest(
            challenge: challenge, name: name, userID: userId)
        request.attestationPreference = ASAuthorizationPublicKeyCredentialAttestationKind.none
        let selection = publicKey["authenticatorSelection"] as? JSObject
        if let verification = selection?["userVerification"] as? String {
            request.userVerificationPreference =
                ASAuthorizationPublicKeyCredentialUserVerificationPreference(rawValue: verification)
        }
        if #available(iOS 17.4, *) {
            request.excludedCredentials = Self.credentialIds(publicKey["excludeCredentials"]).map {
                ASAuthorizationPlatformPublicKeyCredentialDescriptor(credentialID: $0)
            }
        }
        perform(request, for: call)
    }

    @objc func get(_ call: CAPPluginCall) {
        guard let publicKey = call.getObject("publicKey"),
            let rpId = publicKey["rpId"] as? String,
            let challenge = Self.decode(publicKey["challenge"] as? String)
        else {
            call.reject("bad passkey options", "passkey")
            return
        }
        let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(relyingPartyIdentifier: rpId)
        let request = provider.createCredentialAssertionRequest(challenge: challenge)
        if let verification = publicKey["userVerification"] as? String {
            request.userVerificationPreference =
                ASAuthorizationPublicKeyCredentialUserVerificationPreference(rawValue: verification)
        }
        request.allowedCredentials = Self.credentialIds(publicKey["allowCredentials"]).map {
            ASAuthorizationPlatformPublicKeyCredentialDescriptor(credentialID: $0)
        }
        perform(request, for: call)
    }

    private func perform(_ request: ASAuthorizationRequest, for call: CAPPluginCall) {
        DispatchQueue.main.async {
            if self.pending != nil {
                call.reject("a passkey sheet is open already", "passkey")
                return
            }
            self.pending = call
            let controller = ASAuthorizationController(authorizationRequests: [request])
            controller.delegate = self
            controller.presentationContextProvider = self
            self.controller = controller
            controller.performRequests()
        }
    }

    private func takePending() -> CAPPluginCall? {
        let call = pending
        pending = nil
        controller = nil
        return call
    }

    public func presentationAnchor(for controller: ASAuthorizationController) -> ASPresentationAnchor {
        return bridge?.webView?.window ?? ASPresentationAnchor()
    }

    public func authorizationController(
        controller: ASAuthorizationController,
        didCompleteWithAuthorization authorization: ASAuthorization
    ) {
        guard let call = takePending() else { return }
        if let credential = authorization.credential
            as? ASAuthorizationPlatformPublicKeyCredentialRegistration
        {
            guard let attestation = credential.rawAttestationObject else {
                call.reject("no attestation object", "passkey")
                return
            }
            let id = Self.encode(credential.credentialID)
            let response: JSObject = [
                "attestationObject": Self.encode(attestation),
                "clientDataJSON": Self.encode(credential.rawClientDataJSON),
                "transports": ["internal", "hybrid"] as JSArray,
            ]
            call.resolve([
                "id": id, "rawId": id, "type": "public-key",
                "response": response, "extensions": JSObject(),
            ])
        } else if let credential = authorization.credential
            as? ASAuthorizationPlatformPublicKeyCredentialAssertion
        {
            let id = Self.encode(credential.credentialID)
            var response: JSObject = [
                "authenticatorData": Self.encode(credential.rawAuthenticatorData),
                "clientDataJSON": Self.encode(credential.rawClientDataJSON),
                "signature": Self.encode(credential.signature),
            ]
            if credential.userID.isEmpty {
                response["userHandle"] = NSNull()
            } else {
                response["userHandle"] = Self.encode(credential.userID)
            }
            call.resolve([
                "id": id, "rawId": id, "type": "public-key",
                "response": response, "extensions": JSObject(),
            ])
        } else {
            call.reject("unexpected credential", "passkey")
        }
    }

    public func authorizationController(
        controller: ASAuthorizationController, didCompleteWithError error: Error
    ) {
        guard let call = takePending() else { return }
        var code = "passkey"
        if let authError = error as? ASAuthorizationError {
            switch authError.code.rawValue {
            case ASAuthorizationError.Code.canceled.rawValue:
                code = "cancelled"
            case 1006:  // matchedExcludedCredential (iOS 18): this device has one already
                code = "passkey_exists"
            default:
                code = "passkey"
            }
        }
        CAPLog.print("NativeAccount: passkey failed:", error.localizedDescription)
        call.reject(error.localizedDescription, code)
    }

    // MARK: - Session token

    private static let keychainService = "de.onemoreskat.app.session"
    private static let keychainAccount = "session"

    private static var keychainBase: [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: keychainService,
            kSecAttrAccount as String: keychainAccount,
        ]
    }

    @objc func readSession(_ call: CAPPluginCall) {
        var query = Self.keychainBase
        query[kSecReturnData as String] = true
        query[kSecMatchLimit as String] = kSecMatchLimitOne
        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)
        if status == errSecSuccess, let data = item as? Data,
            let token = String(data: data, encoding: .utf8)
        {
            call.resolve(["token": token])
        } else if status == errSecItemNotFound {
            call.resolve(["token": ""])
        } else {
            call.reject("keychain read failed (\(status))", "keychain")
        }
    }

    @objc func writeSession(_ call: CAPPluginCall) {
        SecItemDelete(Self.keychainBase as CFDictionary)
        guard let token = call.getString("token"), !token.isEmpty else {
            call.resolve()
            return
        }
        var item = Self.keychainBase
        item[kSecValueData as String] = Data(token.utf8)
        item[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        let status = SecItemAdd(item as CFDictionary, nil)
        if status == errSecSuccess {
            call.resolve()
        } else {
            call.reject("keychain write failed (\(status))", "keychain")
        }
    }

    // MARK: - base64url

    private static func decode(_ text: String?) -> Data? {
        guard let text = text else { return nil }
        var base64 = text.replacingOccurrences(of: "-", with: "+")
            .replacingOccurrences(of: "_", with: "/")
        while base64.count % 4 != 0 { base64 += "=" }
        return Data(base64Encoded: base64)
    }

    private static func encode(_ data: Data) -> String {
        return data.base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
    }

    /// The ids of a WebAuthn credential list (`[{type, id}]`, ids base64url).
    private static func credentialIds(_ list: JSValue?) -> [Data] {
        guard let list = list as? JSArray else { return [] }
        return list.compactMap { ($0 as? JSObject)?["id"] as? String }.compactMap { decode($0) }
    }
}
