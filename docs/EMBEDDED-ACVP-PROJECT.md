# Validierung von Krypto-Beschleunigern auf Embedded-Geräten mit ACVPeppers

> Projektkurzbeschreibung: Das **ACVPeppers**-Framework nutzen, um NIST-ACVP-Testvektoren
> gegen Krypto-Beschleuniger auf eingebetteten Geräten auszuführen — über eine
> **generische Geräte-Ausführungsschnittstelle**, in die sich jede spezifische
> Zielimplementierung einklinken kann.

## Projekttitel

**Empfohlen:**
**ACVPeppers-probe NIST-ACVP-Konformitätstests für Kryptographysche Funktionen auf Embedded-Geräten**

*Begründung:* behält das „Peppers“-Branding bei, „Probe“ signalisiert den Zugriff auf
ein Hardware-Ziel, und der Untertitel benennt genau die Funktion.

### 5 alternative Titel
1. **ACVPeppers‑Embedded (ACVP‑E)** — *Harness zur kryptografischen Algorithmusvalidierung auf dem Zielgerät*
2. **PepperBridge** — *Host‑zu‑Gerät‑Brücke für NIST‑ACVP‑Testvektoren*
3. **CryptoProbe** — *Validierung von Embedded‑Krypto‑Beschleunigern, gesteuert durch NIST ACVP*
4. **HotWire** — *ACVPeppers mit eingebetteten Krypto‑Engines verdrahten*
5. **ACVP Device Harness (ADH)** — *Ein generisches Backend zum Testen eingebetteter Krypto‑Beschleuniger*

---

## Ziel & Umfang

ACVPeppers so erweitern, dass ein vom ACVP‑Server abgerufener Testfall von einem
**Krypto‑Beschleuniger auf einem eingebetteten Gerät** ausgeführt wird statt von einer
prozessinternen Softwarebibliothek, und die Antwort des Geräts zur
Bestehen/Fehlschlag‑Bewertung an den Server zurückgegeben wird.

Der Kern des Projekts ist eine **generische Schnittstelle** (ein `CryptoBackend`‑artiger
Vertrag, siehe `docs/ANALYSIS.md` §5.2), sodass ein neues Gerät durch **einmalige**
Implementierung dieser Schnittstelle integriert wird — ohne den ACVP‑Protokoll‑Client,
den parallelen Executor oder die Testtyp‑Logik anzufassen.

```
NIST-ACVP-Server ──► ACVPeppers (Host) ──► Generisches Geräte-Backend
                                                │  (Transport: UART/USB/SPI/TCP/Debug-Probe)
                                                ▼
                                        Testagent auf dem Embedded-Gerät
                                                │
                                                ▼
                                        Krypto-Beschleuniger (Prüfling/DUT)
```

**Frühzeitig zu klärende Kernentscheidung:** Bei iterativen Testtypen (MCT = Tausende
verketteter Operationen, LDT = sehr große Eingaben) ist zu entscheiden, ob die Schleife
**auf dem Host** läuft (einfach, aber eine Hin‑ und Rückübertragung pro Operation →
langsam) oder **auf dem Gerät** (schnell, aber der Firmware‑Agent muss die Verkettung
implementieren). Diese Entscheidung bestimmt AP4–AP6.

---

## Arbeitspakete

| AP | Titel | Ziel | Kernaufgaben | Ergebnis |
|----|-------|------|--------------|----------|
| **AP0** | Anforderungen & Auswahl der Zielplattform | Umfang festlegen | Algorithmen & Testtypen im Umfang wählen; Referenzboard + Beschleuniger auswählen; Erfolgskriterien definieren (Server‑„passed“ bei Beispielvektoren) | Einseitige Anforderungsnotiz + Zielliste |
| **AP1** | Generische Geräte‑Backend‑Schnittstelle | Die Kernabstraktion | `CryptoBackend`‑Trait definieren (`supports`, `run_case`, optionales Streaming); Backends je Algorithmus im Executor auflösen | Trait + Registry‑Anbindung (setzt CRYPTO‑2 um) |
| **AP2** | Verallgemeinertes Fall‑/Ergebnismodell | Nicht‑Hash‑Algorithmen unterstützen | Festes `md`‑Ergebnis durch algorithmusunabhängige Feldzuordnung ersetzen (`ct`/`pt`/`tag`/`mac`/…); unbekannte Parserfelder via `#[serde(flatten)]` behalten | Aktualisierte `parser.rs` / `result_format.rs` (CRYPTO‑1) |
| **AP3** | Host‑↔‑Gerät‑Transportabstraktion | Mit beliebiger Zielplattform kommunizieren | `Transport`‑Trait definieren; mindestens eine Implementierung (z. B. seriell/UART oder TCP); füllt das leere `transport/`‑Modul | Transport‑Trait + eine konkrete Transportimplementierung |
| **AP4** | Übertragungsprotokoll & Framing | Anfragen/Antworten kodieren | Kompaktes Anfrage‑/Antwort‑Framing entwerfen (Algorithmus, Testtyp, Eingaben → Ausgaben); versioniert, Embedded‑freundlich (CBOR/TLV/JSON‑Lines) | Protokollspezifikation + Host‑seitiger Codec |
| **AP5** | Geräteseitiger Testagent | Ausführung auf der Zielplattform | Firmware‑Komponente, die eine Anfrage dekodiert, den Beschleuniger‑Treiber aufruft und die Antwort kodiert; Pufferung/Grenzen behandeln | Referenz‑Firmware‑Agent (portabler Kern + HAL‑Anknüpfung) |
| **AP6** | Teststrategie (AFT/MCT/LDT) | Korrekte iterative Tests | AFT implementieren; MCT‑Verkettung (Host vs. Gerät) und LDT‑Streaming über den Transport entscheiden + umsetzen | Funktionierende AFT/MCT/LDT‑Pfade gegen Beispielvektoren |
| **AP7** | Integration der Referenzplattform | Ende‑zu‑Ende nachweisen | Backend für das gewählte Board implementieren; Gerätefähigkeiten → `capabilities.json` abbilden; auf dem ACVP‑**Demo**‑Server validieren | Bestandene Demo‑Session + KAT‑Tests (setzt CRYPTO‑5 um) |
| **AP8** | Automatisierung, CI & Portierungsleitfaden | Wiederholbar machen | Skripte zum Flashen/Ausführen/Sammeln; CI‑Anbindung soweit möglich; Dokumentation „Portierung auf ein neues Gerät“ | CI‑Job(s) + Portierungsleitfaden in `docs/` |

---

## Abhängigkeiten & Bezug zum bestehenden Backlog
- **Baut auf:** CRYPTO‑1 (Ergebnismodell), CRYPTO‑2 (`CryptoBackend`), CRYPTO‑5
  (Secure‑Element‑/Embedded‑Backend) sowie den leeren Nahtstellen `transport/` + `hw_proxy.rs`.
- **Reihenfolge:** Die Credential‑Arbeit ist bereits abgeschlossen; die
  OSS‑Readiness‑Punkte (Lizenz, verbleibende Historie/Rotation) sind unabhängig und
  können parallel laufen.
- **Verantwortliche (Agenten‑Umgebung):** `crypto-backend-integrator` (AP1–AP5),
  `acvp-algorithm-engineer` (AP2, AP6, AP7), `rust-quality-guardian` (Tests/CI über alle).

## Annahmen & nicht im Umfang (initial)
- Zunächst ein Referenzgerät; breite Geräteunterstützung ist eine Folgearbeit.
- Die Einreichung einer produktiven ACVP‑Zertifizierung ist nicht im Umfang — die
  Validierung zielt auf den **Demo**‑Server mit `isSample=true`‑Vektoren.
- Die Firmware‑HAL/‑Treiber für den Beschleuniger wird vom Gerätehersteller
  bereitgestellt; der Agent überbrückt lediglich dorthin.
