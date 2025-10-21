# Rust-Client für die Software-Challenge Germany 2025


Dieses Repository stellt eine Rust-Bibliothek zur Verfügung, die als Vermittlungsschicht (Client <-> Server) für Teilnehmer:innen der **Software Challenge Germany** dienen soll. Es enthält Quellcode und Beispiel-Clients im Ordner `examples`.


---

## Inhaltsverzeichnis

- [Voraussetzungen](#voraussetzungen)  
- [Installation](#installation)  
  - [Als Dependency (für dein Projekt)](#als-dependency-für-dein-projekt)  
  - [Repository lokal bauen](#repository-lokal-bauen)  
- [Erste Schritte / Schnellstart](#erste-schritte--schnellstart)  
- [Beispiele starten](#beispiele-starten)  
- [Konfiguration & Startargumente](#konfiguration--startargumente)  
- [Entwicklung & Tests](#entwicklung--tests)  
- [Stand der Entwicklung / trait-basierte API](#stand-der-entwicklung--trait-basierte-api)  
- [Mitwirken / Contribution](#mitwirken--contribution)  
- [Lizenz](#lizenz)

---

## Voraussetzungen

- Rust toolchain `version: 1.82` — prüfbar mit `rustc --version` und `cargo --version`.  
---

## Installation

### Als Dependency

Füge in deiner `Cargo.toml` z. B. folgendes hinzu:

```toml
[dependencies]
socha = { git = "https://github.com/simoncreates/socha.git" }
```
---


## Stand der Entwicklung — trait-basierte API
Aktuell ist die Bibliothek als reines Kommunikations-Layer implementiert. Eine **trait-basierte API**  ist in Arbeit.


## Mitwirken / Contribution

Beiträge sind willkommen:

1. Issue anlegen (Bug, Feature Request, Frage)  
2. Fork → Branch → PR 
3. Falls du an der trait-Umstellung mitarbeiten willst, mache ein Fork vom in_dev branch
