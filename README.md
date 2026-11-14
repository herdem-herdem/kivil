# Kıvıl

Rust ile geliştirilen küçük bir Türkçe programlama dili yorumlayıcısı.
Paket, terminal komutu ve dosya uzantısı için ASCII `kivil` adı kullanılır.

## Çalıştırma

Rust ve Cargo kurulu olmalıdır. Proje kökünde:

```sh
cargo run -- examples/merhaba.kivil
cargo run -- examples/kosullar_ve_donguler.kivil
cargo run -- examples/islevler.kivil
cargo run -- examples/listeler.kivil
cargo run -- examples/not_hesapla.kivil
```

Kendi dosyanızı `cargo run -- dosya.kivil` ile çalıştırabilirsiniz.
Derlenmiş program için `cargo build --release`, ardından
`./target/release/kivil dosya.kivil` kullanılır. Program tek dosya yolu bekler.
Hatalar standart hata çıkışına dosya, satır ve sütunla yazılır; süreç başarısız
çıkış koduyla biter. Sütunlar UTF-8 baytları yerine Unicode karakterleri sayar.

## Değerler ve işlemler

```text
isim = "Herdem"
puan = 10 + 20 * 3
mesaj = "Merhaba " + isim
yaz(mesaj, "Puanın:", puan)
uygun = puan >= 50 ve puan <= 100
yaz(uygun, degil yanlis, bos)
```

Sayı, metin, mantıksal değer (`dogru` / `yanlis`), boş değer (`bos`) ve liste
desteklenir. Değişkenler yeniden atanabilir ve tür değiştirebilir.
`+ - * /` işlem önceliğiyle çalışır; parantezler önceliği değiştirir.
İki metin `+` ile birleştirilir. Sayı ve metin otomatik olarak birbirine çevrilmez.
Karşılaştırmalar: `== != < <= > >=`. Eşitlik farklı türlerde `yanlis` verir;
sıralama yalnızca sayılar arasındadır. Mantıksal işlemler yalnızca mantıksal
değer kabul eder; `ve` ve `veya` kısa devre yapar. Öncelik düşükten yükseğe:
`veya`, `ve`, `degil`, eşitlik, sıralama, toplama/çıkarma, çarpma/bölme,
tekli `+`/`-`, indeks/metot erişimi. Zincirli sıralama yerine `a < b ve b < c` yazılır.

Hesaplamalar `f64` kullanır: büyük tam sayılarda ve ondalıklı işlemlerde
kayan nokta hassasiyet sınırları vardır. Sıfıra bölme ve sonsuz sonuçlar hata verir.
Kaynakta sayılar ondalık rakamlarla, örneğin `12` ve `1.25`, yazılır.

## Satırlar, metinler ve bloklar

`eger` ve `degilse eger` koşulları zorunlu olarak `(...)` içinde yazılır.
Her komut ayrı satırdadır; `;` kullanılmaz. Bloklar `{ }` ile sınırlanır;
girinti okunabilirlik içindir. Boş bir blok `{}` biçiminde yazılabilir.
`#` ile satır sonuna kadar yorum yazılır. Tırnak içindeki `#` normal metindir.
Metinler çift tırnakla yazılır; `\n`, `\t`, `\"`, `\\` kaçışları desteklenir.
Ters eğik çizgi yazdırmak için `\\` kullanılır. İfadeler, argüman listeleri ve
liste sabitleri şu an tek satırda yazılır.

```text
eger (puan >= 50) {
    yaz("Geçtin")
} degilse eger (puan >= 40) {
    yaz("Yaklaştın")
} degilse {
    yaz("Tekrar dene")
}
sayac = 0
iken sayac < 3 {
    yaz(sayac)
    sayac = sayac + 1
}
tekrarla 2 {
    yaz("Kıvıl!")
}
```

Koşullar mantıksal değer ister. `iken` koşulu her turda hesaplanır.
`tekrarla` sayısı döngüye girerken bir kez hesaplanır ve platformun desteklediği
aralıkta negatif olmayan tam sayı olmalıdır. Koşul/döngü blokları kendi değişken
kapsamını açmaz. Tek başına ifadeler üst düzeyde sonucu gösterir; atamalar göstermez.

## Girdi ve yerleşik işlevler

```text
isim = girdi("Adın: ")
yas = sayi(girdi("Yaşın: "))
yaz(isim, "gelecek yıl", yas + 1)
yaz("Yaş: " + metin(yas))
```

- `yaz(...)`: Değerleri aralarında boşlukla, sonunda yeni satırla yazar; `bos` döndürür.
- `girdi()` / `girdi("Soru: ")`: Bir satır okur ve metin döndürür. Soru hemen
  gönderilir; yalnızca satır sonu kaldırılır. Beklenmedik giriş sonu hata verir.
- `sayi(deger)`: Sayıyı korur veya metni sonlu bir sayıya dönüştürür; geçersiz girdi hata verir.
- `metin(deger)`: Değerin yazdırılabilir metin karşılığını döndürür.

## Fonksiyonlar

```text
islev topla(a, b) {
    dondur a + b
}
yaz(topla(10, 20))
```

Parametreler ve fonksiyon içindeki atamalar yereldir. Fonksiyon kendi yerel
kapsamını ve global değişkenleri okuyabilir; çağıran fonksiyonun yerel
kapsamını okuyamaz. Yerel atama global değişkeni değiştirmez. `dondur`, iç içe
koşul/döngülerden de fonksiyonu hemen bitirir. Değer döndürmeyen işlev `bos`
döndürür. Fonksiyon gövdesindeki çıplak ifadeler otomatik yazdırılmaz; `yaz`
kullanılır. Özyineleme desteklenir, iç içe çağrı sınırı 64'tür.

Fonksiyon tanımı çağrıdan önce çalışmış olmalıdır. Yerleşik işlevler yeniden
tanımlanamaz. İç içe fonksiyon tanımı ve fonksiyonların değer olarak aktarımı
henüz desteklenmez. Parametre adları tekrarlanamaz; argüman sayısı doğrulanır.

## Listeler

```text
sayilar = [10, 20, 30]
sayilar[1] = 99
sayilar.ekle(40)
yaz(sayilar[0], sayilar.uzunluk(), sayilar)
```

İndeksler sıfırdan başlar. Negatif, kesirli ve sınır dışı indeksler hata verir.
`ekle(deger)` listeyi değiştirir ve `bos` döndürür; `uzunluk()` sayı döndürür.
Farklı türlerde elemanlar ve iç içe listeler desteklenir. Listeler paylaşılır:
`b = a` atamasından sonra `b[0] = 5`, `a` üzerinden de görünür. Aynısı liste
parametreleri için geçerlidir. Liste eşitliği elemanlara göre karşılaştırılır.
Döngüsel listeler (doğrudan veya dolaylı kendi içine ekleme) reddedilir.

## Geliştirme ve doğrulama

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

`lexer.rs` konumlu token'lar, `parser.rs` ifade/komut ağaçları,
`interpreter.rs` değerler, kapsamlar ve çalıştırmayı yönetir.
`diagnostic.rs` ortak hata ve konum tiplerini içerir.
Standart kütüphane dışında bağımlılık kullanılmaz.
