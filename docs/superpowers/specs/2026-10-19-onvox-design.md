# Onvox — Tasarım Belgesi

Tarih: 2026-10-19
Depo: `github.com/wenlar/onvox` · Yerel: `D:\Work\onvox` · Alan adı: `onvox.app`

## Ne ve neden

Onvox, Kick/Twitch yayıncıları için ücretsiz bir Windows programı. Spotify gibi arka planda,
sistem tepsisinde durur. Yayıncı oyun oynarken ya da yayın yaparken bir kısayol tuşuna basar,
kendi ChatGPT veya Grok hesabının **kendi sesli modu** ile doğal, araya girilebilir şekilde
konuşur; tekrar basınca konuşma kapanır ve yayına devam eder. AI'ın sesi yayına gider,
izleyiciler de duyar.

Hedef kitle teknik olmayan yayıncılar: API anahtarı, hesap kurulumu, ayar dosyası yok.
"Giriş yap, tuşa bas, konuş."

## Hedefler

- Kısayol tuşu ile sesli sohbeti aç/kapat; oyun penceresi odağını **asla** kaybetmez.
- Yayıncının mevcut aboneliğini kullanır (ChatGPT Free/Plus, Grok Free/ücretli), ek maliyet yok.
- Doğal ChatGPT/Grok sesi (sitelerin kendi sesli modu), sıralı TTS değil.
- Küçük kurulum dosyası, imzalı installer, otomatik güncelleme.

## Hedef olmayanlar (v1)

- Claude (sonraki aşama; claude.ai'de web sesli mod beta olarak var).
- API anahtarı desteği, ücretli plan, Wenlar hesabı / sunucusu.
- İzleyicilerin sohbetten AI'a soru sorması.
- Ayrı bir ayarlar penceresi (her şey tepsi menüsünde).
- macOS / Linux.

## Anti-hedefler (teknik olarak çalışsa bile başarısızlık sayılır)

- Program oyunun önüne gelir, odak çalar ya da oyunda takılma/kare düşüşü yaratır.
- Kısayol tuşu oyunun kullandığı bir tuşu bozar ve yayıncı bunu değiştiremez.
- Program anti-cheat sistemlerini tetikler ve yayıncının oyun hesabı risk altına girer.
- Yayıncının kurulum için API anahtarı, terminal ya da teknik adım uygulaması gerekir.
- Hata durumunda oyunun üstüne pencere/diyalog açılır.

## Kısıtlar ve bilinen riskler

- **Resmi yol yok:** ChatGPT ve Grok abonelikleri üçüncü taraf programlara sesli mod erişimi
  sunmuyor. Tek yol, sitelerin kendi web sayfasını gizli bir WebView2 penceresinde açıp sesli
  mod düğmesine programın basması.
- **Kırılganlık:** site arayüzü değişince bozulur → hızlı güncelleme dağıtımı şart (otomatik
  güncelleme bu yüzden v1'de).
- **Kullanım şartları:** OpenAI şartları API dışı otomatik veri çekmeyi yasaklıyor; Onvox veri
  çekmiyor, sadece düğmeye basıyor, ses gerçek kullanıcıdan geliyor — yine de gri bölge.
  xAI şartları doğrulanamadı. Yayıncıya ilk kurulumda kısa bir uyarı gösterilmeli (metin: builder).
- **Limitler:** ChatGPT Plus ~3 saat/gün ses; Grok limitleri belirsiz ve değişken. Limit dolunca
  hata durumu (aşağıda).
- **Kanıt:** [Grok-Desktop](https://github.com/AnRkey/Grok-Desktop) (Electron) grok.com sesli
  modunun gömülü tarayıcıda çalıştığını gösteriyor. ChatGPT için ve WebView2 için kanıt yok →
  ilk iş bir doğrulama denemesi (aşağıda).

## Teknoloji (kararlaştırıldı)

- **Rust + Tauri v2.** Gezik ile aynı dil. Slint değil: Slint web sayfası barındıramaz ve bu
  programın kendi arayüzü neredeyse yok.
- WebView2 (Windows 11'de hazır), Tauri tray + global-shortcut eklentileri, Tauri bundler
  (NSIS/MSI), Windows kod imzalama, Tauri updater.
- Kurulum boyutu küçük olmalı (hedef: birkaç MB).
- **Kod imzalama (kararlaştırıldı):** Gezik'te kullanılan Azure Trusted Signing (Artifact
  Signing) hesabı ve sertifika profili. exe, NSIS kurulum dosyası ve kaldırıcı imzalanır.

## Release iş akışı (Gezik'in `.github/workflows/release.yml`'inden)

Gezik'ten aynen alınacaklar:

- `v<sürüm>` etiketi push'lanınca derlenir ve **taslak** (draft) GitHub Release açılır; elle
  çalıştırma (`workflow_dispatch`) sadece derleyip artifact yükler.
- `version` işi: etiket `Cargo.toml` sürümüyle tutmazsa iş durur. Onvox'ta `tauri.conf.json`
  sürümü de aynı olmalı (ya da oradan Cargo sürümü okunmalı).
- `-` içeren sürüm (`0.1.0-rc.1`) `--prerelease` olarak işaretlenir.
- Azure girişi OIDC ile: `release` ortamına (environment) bağlı federe kimlik, `id-token: write`;
  `vars.AZURE_CLIENT_ID / AZURE_TENANT_ID / AZURE_SUBSCRIPTION_ID / AZURE_SIGNING_ENDPOINT /
  AZURE_SIGNING_ACCOUNT / AZURE_CERT_PROFILE`. Değişkenler yoksa imzalama atlanır, imzasız
  derleme yine çalışır.
- Zaman damgası `http://timestamp.acs.microsoft.com`, SHA256.
- `SHA256SUMS.txt`, sürümü sabitlenmiş (pinned) action'lar, `--generate-notes`.

Onvox'ta farklı olanlar:

- Sadece Windows işi (macOS/Linux yok).
- Paket zip değil, `cargo tauri build` ile NSIS kurulum dosyası.
- İmzalama ayrı bir adım olarak değil, Tauri'nin `bundle.windows.signCommand` ayarıyla yapılır
  (ör. `trusted-signing-cli`, `azure/login` sonrası Azure CLI kimliğiyle). Sebep: Tauri hem
  içteki exe'yi hem kurulum dosyasını paketlerken imzalamalı, ve güncelleme imzası (`.sig`)
  Authenticode imzalı kurulum dosyası üzerinden hesaplanmalı; sonradan imzalanırsa `.sig` bozulur.
- Release'e ek olarak `.sig` ve `latest.json` yüklenir. Taslak release yayınlanana kadar
  `releases/latest/download/latest.json` onu görmez, yani güncelleme yayıncılara ancak
  release elle yayınlanınca gider (Gezik'teki taslak → yayın adımıyla aynı).
- `TAURI_SIGNING_PRIVATE_KEY` (+ parolası) GitHub Secrets'ta.

**Yapılması gereken (Azure tarafı, Teoman):** federe kimlik depoya bağlı olduğu için Azure'daki
uygulama kaydına `wenlar/onvox` için yeni bir federated credential eklenmeli
(konu: `repo:wenlar/onvox:environment:release`) ve imzalama hesabında aynı rol geçerli olmalı.
Sertifika profili aynı kalır.

## Kullanıcı deneyimi

### İlk kurulum

1. Kurulumdan sonra program bir kerelik **görünür** bir pencere açar: sağlayıcı seçimi
   (ChatGPT / Grok) ve seçilen sitenin giriş sayfası. Program öne yalnızca bu adımda gelir.
2. Yayıncı kendi hesabıyla giriş yapar; mikrofon izni bu adımda verilir.
3. Pencere kapanır, program tepsiye iner. Windows açılışında otomatik başlar. *(benim kararım)*

Giriş bilgileri/oturum WebView2 profilinde kalır; her sağlayıcı ayrı profil. *(benim kararım)*

### Yayın sırasında

- **Kısayol tuşuna basış (kapalıyken):** "açıldı" bip sesi → tepsi ikonu "aktif" rengine döner →
  gizli sayfada sesli mod başlar → OBS göstergesi (varsa) belirir.
- AI'ın sesi Windows varsayılan çıkış cihazına gider; OBS masaüstü sesi bunu yakalar, izleyiciler
  duyar. Yayıncının sorusu zaten kendi mikrofonundan yayına gider.
- **Tekrar basış:** farklı bir "kapandı" bip sesi → sesli mod kapanır → mikrofon bırakılır →
  ikon ve gösterge normale döner.
- Bip sesleri de varsayılan çıkışa gider, izleyiciler de duyar (bilinçli; bir işaret olarak).
- Hiçbir adımda pencere görünmez, odak değişmez.

### Sohbet sürekliliği (kararlaştırıldı)

- Program çalıştığı sürece her tuş basışı **aynı sohbete** devam eder (gece yarısını geçen
  yayınlar dahil).
- Program yeniden başlayınca (bilgisayar kapanıp açılınca) **yeni sohbet**.
- Tepsi menüsünden "Yeni sohbet başlat" ile elle yeni sohbet.

### Kısayol tuşu

- Varsayılan, oyunlarda nadir kullanılan bir kombinasyon (öneri: `Ctrl+Alt+Space`). *(benim kararım)*
- Tepsi menüsünden değiştirilebilir.
- Tauri global-shortcut (`RegisterHotKey`) kullanılır; bu kombinasyonu oyundan alır, o yüzden
  nadir kombinasyon + değiştirilebilirlik çözüm sayılır. Düşük seviye klavye kancası
  (`WH_KEYBOARD_LL`) **kullanılmaz**: anti-cheat sistemleri (Vanguard, EAC, BattlEye) bunu
  işaretleyebilir ve yayıncının hesabı banlanabilir. *(benim kararım)*
- Oyunun tam ekran / exclusive fullscreen modunda da çalışmalı.

### Tepsi menüsü

- Sağlayıcı: ChatGPT / Grok
- Yeni sohbet başlat
- Kısayol tuşunu değiştir
- OBS gösterge adresini kopyala
- Yeniden giriş yap
- Durum satırı (son hata varsa burada)
- Çıkış

### OBS göstergesi (isteğe bağlı, v1'de) *(benim kararım)*

Program yerel bir adres sunar (ör. `http://localhost:<port>/overlay`). Yayıncı isterse OBS'e
Tarayıcı Kaynağı olarak ekler; AI dinlerken/konuşurken küçük bir gösterge belirir, kapanınca
kaybolur. Eklemeyen yayıncıyı hiç etkilemez. Yalnızca `localhost`'tan erişilebilir olmalı.

### Hata durumları *(benim kararım)*

Oturum düştü, ses limiti doldu, sesli mod düğmesi bulunamadı, mikrofon yok:
ayrı bir "hata" bip sesi + kırmızı tepsi ikonu + ayrıntı tepsi menüsündeki durum satırında.
Oturum düştüyse "Yeniden giriş yap" görünür girişi açar — ama **yalnızca yayıncı menüden
tıklayınca**, kendiliğinden asla.

## İlk adım: doğrulama denemesi (spike)

Plan yazılmadan önce atılacak bir deneme, şu soruların cevabını vermeli:

1. Tauri v2'de gizli (görünmez, odaksız) bir WebView2 penceresinde chatgpt.com ve grok.com'a
   giriş yapılmış oturumla sesli mod başlatılabiliyor mu?
2. Mikrofon izni WebView2 `PermissionRequested` olayıyla otomatik verilebiliyor mu?
3. Gizli pencere arka planda kısılmadan (throttling) ses alıp verebiliyor mu?
4. Sesli mod düğmesine programatik basış (DOM üzerinden) güvenilir mi?
5. Genel kısayol tuşu tam ekran bir oyun açıkken tetikleniyor mu?
6. Sitelerin bot kontrolü gömülü WebView2 girişini engelliyor mu?

Bir sağlayıcı için "hayır" çıkarsa v1 **çalışan tek sağlayıcıyla çıkabilir** (kararlaştırıldı);
çalışmayan sağlayıcı sonraki sürüme kalır. Sağlayıcı yapısı yine de ikinci sağlayıcının
sonradan eklenebileceği şekilde kurulur. Her iki sağlayıcı için de "hayır" çıkarsa ya da
1, 2, 3 veya 5. sorular genel olarak "hayır" ise tasarım buraya geri döner.

## Otomatik güncelleme (kararlaştırıldı: Gezik'e benzer)

GitHub'da `wenlar/onvox` altında bir release yayınlanınca kurulu uygulamalar güncellenir.
Davranış Gezik'inkini izler (`wenlar/gezik` → `docs/superpowers/notes/parts/update.md`):

- Açılıştan kısa süre sonra ve ~20 saatte bir GitHub Releases kontrol edilir; taslaklar atlanır,
  kararlı kanalda ön sürümler (`prerelease` / `-rc` etiketleri) atlanır.
- Yeni sürüm arka planda indirilir ve doğrulanır.
- Kurulum **asla sesli konuşma sırasında** olmaz: program kapanırken ya da tepsi menüsünden
  "Güncelle ve yeniden başlat" ile. Tepsi menüsünde "X sürümü hazır" satırı görünür; oyunun
  üstüne bildirim/pencere açılmaz.
- Güncellemeden sonraki ilk açılışta tepsi menüsünde bir kez "X sürümüne güncellendi · Yenilikler".

Mekanizma *(benim kararım)*: Gezik'in elle yazılmış güncelleyicisini taşımak yerine Tauri'nin
kendi updater eklentisi kullanılır. Endpoint
`https://github.com/wenlar/onvox/releases/latest/download/latest.json`; release'e NSIS kurulum
dosyası, `.sig` imzası ve `latest.json` CI tarafından eklenir. Tauri updater ayrı bir imza
anahtarı (minisign) ister; özel anahtar GitHub Secrets'ta tutulur. Bu, Windows kod imzalamadan
bağımsızdır.

## Site değişikliklerini fark etme *(benim kararım)*

- Wenlar sunucusu yok, telemetri yok. Kaynak: kullanıcı bildirimleri.
- Tepsi menüsünde "Sorun bildir" → GitHub Issues sayfasını tarayıcıda açar (sürüm ve sağlayıcı
  bilgisi bağlantıya eklenmiş olarak).
- Site başına düğme seçicileri, uygulama içine gömülü varsayılanlarla birlikte GitHub'da duran
  uzak bir dosyadan da okunur; küçük arayüz değişiklikleri tam sürüm çıkarmadan düzeltilebilir.

## Builder'a bırakılanlar

- Site başına sesli mod düğmesini bulma/basma yöntemi ve bunların site değişikliğine karşı
  dayanıklılığı (seçicilerin kod dışında, güncellenebilir tutulması dahil).
- Bip sesleri, ikon tasarımları, OBS göstergesinin görünümü.
- OBS göstergesi için port seçimi ve iletişim yöntemi.
- Tepsi menüsü metinleri ve kullanım şartları uyarısının metni.
- Proje yapısı, test yaklaşımı.
