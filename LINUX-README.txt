MALA MATEMATIKA - Linux x64

Raspakuj tar.gz arhivu i pokreni mala-matematika dvoklikom.
Ako upravljac fajlovima ne pokrece izvrsne fajlove, otvori terminal
u raspakovanom folderu i pokreni:
  ./mala-matematika
Ako je pri kopiranju izgubljena dozvola za izvrsavanje:
  chmod +x mala-matematika

Za igranje nisu potrebni Rust, Bevy ili poseban assets direktorijum.
Font za srpsku cirilicu je ugradjen u samu igricu.
Potreban je Linux x64 sa X11 ili Wayland grafickim okruzenjem
i grafickim drajverom koji podrzava Bevy/wgpu.
Paket koristi sistemske biblioteke racunara na kome je napravljen.
Na starijim distribucijama moze biti potrebno ponovno kompajliranje
iz izvornog koda na toj distribuciji (bash scripts/build-linux.sh).

Odgovori: misem ili tasterima 1, 2 i 3 (polozaj ponudjenog odgovora).
Posle tacnog odgovora sledeci zadatak stize automatski za 2 sekunde.
Promena vrste racuna zapocinje novu igru i resetuje bodove.
Napredak se ne cuva nakon zatvaranja prozora.

Izaberi "Vezba do 10" za neograniceno vezbanje bez nivoa.
Brojevi i rezultati ostaju do 10, uz sabiranje, oduzimanje ili oba.
"Sa nivoima" zadrzava tri nivoa do 10, 15 i 20.
Promena rezima resetuje bodove i broj resenih zadataka.

Liberation Sans font je ukljucen prema licenci u LICENSE-Liberation.txt.
Prilikom deljenja paketa zadrzi taj fajl zajedno sa igrom.
