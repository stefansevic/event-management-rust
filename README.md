# Event Management System  
Mikroservisna aplikacija za upravljanje događajima

## 1. Uvod
Ovaj projekat predstavlja implementaciju mikroservisne aplikacije za upravljanje događajima
(konferencije, radionice, koncerti, meetupi). Sistem omogućava korisnicima registraciju i
autentifikaciju, pregled dostupnih događaja, prijavu na događaje, kao i upravljanje kapacitetima
i rezervacijama.

Backend sistema je implementiran u programskom jeziku Rust, u skladu sa principima
mikroservisne arhitekture. Fokus projekta je na pravilnoj podeli odgovornosti servisa,
komunikaciji između njih i primeni savremenih arhitektonskih obrazaca, dok izgled
korisničkog interfejsa nije u fokusu ocenjivanja.

## 2. Problem koji se rešava
Organizacija događaja često zahteva rad sa velikim brojem korisnika, prijava i ograničenim
kapacitetima. Monolitna rešenja otežavaju skaliranje sistema, uvođenje novih funkcionalnosti
i održavanje koda.

Glavni problemi koje sistem adresira su:
- nepostojanje centralizovane evidencije događaja i prijava
- neefikasno upravljanje kapacitetima događaja
- nejasna autorizacija i kontrola pristupa
- otežano proširenje sistema novim funkcionalnostima

## 3. Predlog rešenja
Predloženo rešenje je web aplikacija zasnovana na mikroservisnoj arhitekturi, gde je svaki
servis zadužen za jasno definisan domen problema i poseduje sopstvenu bazu podataka.
Servisi međusobno komuniciraju putem REST API-ja.

Sistem je projektovan tako da omogući:
- modularnost i lakše održavanje
- nezavisno skaliranje servisa
- postepeno proširivanje funkcionalnosti

## 4. Arhitektura sistema
Aplikacija se sastoji od sledećih mikroservisa:

### 4.1 Auth Service
Namena: Autentifikacija i autorizacija korisnika.

Funkcionalnosti:
- registracija korisnika
- prijava korisnika
- izdavanje i validacija JWT tokena
- upravljanje ulogama (korisnik, organizator, administrator)

Tehnologije:
- Rust (Actix-web ili Axum)
- PostgreSQL
- JWT

### 4.2 User Service
Namena: Upravljanje korisničkim profilima.

Funkcionalnosti:
- CRUD operacije nad korisnicima
- čuvanje osnovnih podataka o korisnicima i organizatorima

Tehnologije:
- Rust
- PostgreSQL

### 4.3 Event Service
Namena: Upravljanje događajima.

Funkcionalnosti:
- kreiranje, izmena i brisanje događaja
- definisanje kapaciteta događaja
- pretraga i filtriranje događaja

Tehnologije:
- Rust
- PostgreSQL

### 4.4 Registration / Ticket Service
Namena: Upravljanje prijavama i rezervacijama za događaje.

Funkcionalnosti:
- prijava korisnika na događaj
- otkazivanje prijave
- kontrola popunjenosti kapaciteta
- generisanje karata za događaje

Tehnologije:
- Rust
- PostgreSQL

### 4.5 API Gateway (opciono)
API Gateway predstavlja jedinstvenu ulaznu tačku ka backend sistemu i zadužen je za:
- prosleđivanje zahteva ka odgovarajućim servisima
- validaciju JWT tokena
- agregaciju podataka iz više servisa (API composition)

## 5. Baze podataka
Svaki mikroservis poseduje sopstvenu bazu podataka u skladu sa principima mikroservisne
arhitekture:

- Auth Service – PostgreSQL
- User Service – PostgreSQL
- Event Service – PostgreSQL
- Registration Service – PostgreSQL

## 6. Bezbednost
- JWT autentifikacija
- Role-based autorizacija
- validacija zahteva na nivou servisa i API Gateway-a
- zaštita privatnih endpoint-a

## 7. Planirane dodatne funkcionalnosti
- generisanje QR koda za ulaznice
- kontejnerizacija servisa korišćenjem Docker alata
- osnovne analitike (broj prijava po događaju)
- proširenje sistema ka asinhronoj komunikaciji između servisa

## 8. Tehnologije
- Rust (obavezno)
- Actix-web ili Axum
- PostgreSQL
- Docker (planirano)
- Frontend web aplikacija (nije u fokusu ocenjivanja)

## 9. Ciljani broj poena
Projektni zadatak se radi za maksimalan broj poena, uz mogućnost proširenja u skladu
sa sugestijama asistenata.

## 10. Zaključak
Projekat demonstrira primenu mikroservisne arhitekture u realnom problemu upravljanja
događajima. Sistem je modularan, skalabilan i projektovan tako da omogućava dalji razvoj
i eventualno proširenje u diplomski rad.
