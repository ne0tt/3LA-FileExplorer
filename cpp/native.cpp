#include "native.h"

#include <QClipboard>
#include <QCoreApplication>
#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QFont>
#include <QFontDatabase>
#include <QGuiApplication>
#include <QIcon>
#include <QImageReader>
#include <QMimeData>
#include <QMimeDatabase>
#include <QQmlApplicationEngine>
#include <QQuickImageProvider>
#include <QProcess>
#include <QQuickStyle>
#include <QStandardPaths>
#include <QThread>
#include <QThreadPool>
#include <QUrl>

#include <atomic>

namespace native {

static QString qstr(rust::Str s)
{
    return QString::fromUtf8(s.data(), static_cast<qsizetype>(s.size()));
}

static rust::String rstr(const QString& s)
{
    const QByteArray b = s.toUtf8();
    return rust::String(b.constData(), static_cast<size_t>(b.size()));
}

void set_quick_style(rust::Str style)
{
    QQuickStyle::setStyle(qstr(style));
}

void set_app_font(rust::Str family, double point_size)
{
    QFont f(qstr(family));
    f.setPointSizeF(point_size);
    f.setHintingPreference(QFont::PreferNoHinting);
    QGuiApplication::setFont(f);
}

rust::String register_font_data(rust::Slice<const uint8_t> data)
{
    const QByteArray bytes(reinterpret_cast<const char*>(data.data()), static_cast<qsizetype>(data.size()));
    const int id = QFontDatabase::addApplicationFontFromData(bytes);
    const QStringList families = QFontDatabase::applicationFontFamilies(id);
    return rstr(families.isEmpty() ? QString() : families.first());
}

// image://icon/<name> -> freedesktop theme icon (qt6ct sets the theme).
class ThemeIconProvider : public QQuickImageProvider {
public:
    ThemeIconProvider() : QQuickImageProvider(QQuickImageProvider::Pixmap) {}

    QPixmap requestPixmap(const QString& id, QSize* size, const QSize& requested) override
    {
        const int s = requested.width() > 0 ? requested.width() : 32;
        QIcon icon = QIcon::fromTheme(id);
        if (icon.isNull() && QFile::exists(id))
            icon = QIcon(id);
        if (icon.isNull())
            icon = QIcon::fromTheme("application-x-executable");
        const QPixmap pm = icon.pixmap(s, s);
        if (size)
            *size = pm.size();
        return pm;
    }
};

// image://thumb/<path> -> thumbnail for a picture or video: a freedesktop
// cache hit (tumbler/other managers), otherwise a scaled decode (pictures) or
// an ffmpeg frame grab (videos), written back to the cache so each file is
// processed once. Failures leave a marker in fail/ so they aren't retried.
namespace thumbs {

static void store(QImage img, const QString& dir, const QString& name, const QByteArray& uri, qint64 mtime, qint64 bytes)
{
    if (!QDir().mkpath(dir))
        return;
    QFile::setPermissions(dir, QFileDevice::ReadOwner | QFileDevice::WriteOwner | QFileDevice::ExeOwner);
    img.setText("Thumb::URI", QString::fromLatin1(uri));
    img.setText("Thumb::MTime", QString::number(mtime));
    img.setText("Thumb::Size", QString::number(bytes));
    img.setText("Software", "3LA File Explorer");
    // Write under a temporary name so readers never see a partial PNG.
    const QString tmp = dir + name + "." + QString::number(QCoreApplication::applicationPid()) + ".tmp";
    if (img.save(tmp, "PNG")) {
        QFile::setPermissions(tmp, QFileDevice::ReadOwner | QFileDevice::WriteOwner);
        QFile::remove(dir + name);
        QFile::rename(tmp, dir + name);
    } else {
        QFile::remove(tmp);
    }
}

// One keyframe, scaled to fit `bucket`. Seeks a few seconds in to skip black
// intros; short clips fall back to the first frame.
static QImage videoFrame(const QString& path, int bucket)
{
    static const bool haveFfmpeg = !QStandardPaths::findExecutable("ffmpeg").isEmpty();
    if (!haveFfmpeg)
        return QImage();
    const QString scale = QString("scale=%1:%1:force_original_aspect_ratio=decrease").arg(bucket);
    for (const char* at : { "5", "0" }) {
        QProcess p;
        p.start("ffmpeg", { "-nostdin", "-v", "error", "-threads", "1", "-skip_frame", "nokey",
                            "-ss", at, "-i", path, "-frames:v", "1", "-vf", scale,
                            "-f", "image2pipe", "-vcodec", "png", "-" });
        if (!p.waitForFinished(15000)) {
            p.kill();
            p.waitForFinished();
            return QImage();
        }
        QImage img;
        if (img.loadFromData(p.readAllStandardOutput(), "PNG"))
            return img;
    }
    return QImage();
}

static QImage make(const QString& id, const QSize& requested, const std::atomic<bool>& cancelled)
{
    const int s = requested.width() > 0 ? requested.width() : 128;
    const QString decoded = QUrl::fromPercentEncoding(id.toUtf8());
    const QString path = decoded.startsWith('/') ? decoded : "/" + decoded;
    const QByteArray uri = QUrl::fromLocalFile(path).toEncoded();
    const QString hash = QString::fromLatin1(
        QCryptographicHash::hash(uri, QCryptographicHash::Md5).toHex()) + ".png";
    const QString cache = QStandardPaths::writableLocation(QStandardPaths::GenericCacheLocation) + "/thumbnails/";
    const QFileInfo fi(path);
    const qint64 mtime = fi.lastModified().toSecsSinceEpoch();
    // A thumbnail is only valid for the file version it was made from.
    const auto fresh = [&](const QImage& img) {
        return static_cast<qint64>(img.text("Thumb::MTime").toDouble()) == mtime;
    };

    // Smallest cached size that still covers the request comes first.
    const int bucket = s <= 128 ? 128 : s <= 256 ? 256 : 512;
    const QStringList sizes = { "normal/", "large/", "x-large/" };
    const int start = bucket == 128 ? 0 : bucket == 256 ? 1 : 2;
    const QString own = sizes[start];
    QStringList dirs = sizes.mid(start);
    for (int i = start - 1; i >= 0; --i)
        dirs << sizes[i];

    QImage img;
    for (const QString& dir : dirs) {
        if (img.load(cache + dir + hash) && fresh(img))
            break;
        img = QImage();
    }
    if (img.isNull()) {
        // Some thumbnailer already failed on this version of the file.
        for (const QFileInfo& f : QDir(cache + "fail").entryInfoList(QDir::Dirs | QDir::NoDotAndDotDot)) {
            QImage marker;
            if (marker.load(f.filePath() + "/" + hash) && fresh(marker))
                return QImage();
        }
        // The view may have scrolled past this item while it was queued.
        if (cancelled || !fi.isFile() || path.startsWith(cache))
            return QImage();
        static const QMimeDatabase db;
        const bool video = db.mimeTypeForFile(path, QMimeDatabase::MatchExtension).name().startsWith("video/");
        bool cacheIt = video;
        if (video) {
            img = videoFrame(path, bucket);
        } else {
            QImageReader r(path);
            r.setAutoTransform(true);
            const QSize full = r.size();
            cacheIt = full.isValid() && (full.width() > bucket || full.height() > bucket);
            if (cacheIt)
                r.setScaledSize(full.scaled(bucket, bucket, Qt::KeepAspectRatio));
            img = r.read();
        }
        if (cancelled)
            return QImage();
        if (!img.isNull() && cacheIt)
            store(img, cache + own, hash, uri, mtime, fi.size());
        else if (img.isNull() && fi.isReadable())
            store(QImage(1, 1, QImage::Format_ARGB32), cache + "fail/3la-fileexplorer/", hash, uri, mtime, fi.size());
    }
    if (!img.isNull() && (img.width() > s || img.height() > s))
        img = img.scaled(s, s, Qt::KeepAspectRatio, Qt::SmoothTransformation);
    return img;
}

class Response : public QQuickImageResponse, public QRunnable {
public:
    Response(const QString& id, const QSize& requested) : m_id(id), m_requested(requested) { setAutoDelete(false); }

    void run() override
    {
        if (!m_cancelled)
            m_image = make(m_id, m_requested, m_cancelled);
        emit finished();
    }

    void cancel() override { m_cancelled = true; }

    QQuickTextureFactory* textureFactory() const override
    {
        return QQuickTextureFactory::textureFactoryForImage(m_image);
    }

private:
    QString m_id;
    QSize m_requested;
    QImage m_image;
    std::atomic<bool> m_cancelled { false };
};

// A small pool, so one slow video doesn't hold up every other thumbnail
// (a plain async provider handles requests one at a time).
class Provider : public QQuickAsyncImageProvider {
public:
    Provider() { m_pool.setMaxThreadCount(qBound(2, QThread::idealThreadCount() / 2, 4)); }
    ~Provider() override { m_pool.clear(); m_pool.waitForDone(); }

    QQuickImageResponse* requestImageResponse(const QString& id, const QSize& requested) override
    {
        auto* r = new Response(id, requested);
        m_pool.start(r);
        return r;
    }

private:
    QThreadPool m_pool;
};

}

void install_image_providers(QQmlApplicationEngine& engine)
{
    engine.addImageProvider("icon", new ThemeIconProvider);
    engine.addImageProvider("thumb", new thumbs::Provider);
}

bool has_root_objects(const QQmlApplicationEngine& engine)
{
    return !engine.rootObjects().isEmpty();
}

// Mirrors the GNOME/Thunar clipboard format so cut/copy interoperates with
// Thunar, Nautilus and Dolphin.
void set_clipboard_uris(const rust::Vec<rust::String>& uris, bool cut)
{
    QList<QUrl> urls;
    QByteArray gnome = cut ? "cut" : "copy";
    QStringList plain;
    for (const auto& u : uris) {
        const QString s = QString::fromUtf8(u.data(), static_cast<qsizetype>(u.size()));
        const QUrl url(s);
        urls << url;
        gnome += "\n" + url.toEncoded();
        plain << url.toLocalFile();
    }
    auto* mime = new QMimeData;
    mime->setUrls(urls);
    mime->setData("x-special/gnome-copied-files", gnome);
    mime->setData("application/x-kde-cutselection", cut ? "1" : "0");
    mime->setText(plain.join('\n'));
    QGuiApplication::clipboard()->setMimeData(mime);
}

rust::Vec<rust::String> clipboard_uris()
{
    rust::Vec<rust::String> out;
    const QMimeData* mime = QGuiApplication::clipboard()->mimeData();
    if (!mime)
        return out;
    if (mime->hasFormat("x-special/gnome-copied-files")) {
        const auto lines = mime->data("x-special/gnome-copied-files").split('\n');
        for (qsizetype i = 1; i < lines.size(); ++i) {
            const QByteArray l = lines[i].trimmed();
            if (!l.isEmpty())
                out.push_back(rstr(QUrl::fromEncoded(l).toString(QUrl::FullyEncoded)));
        }
        return out;
    }
    for (const QUrl& u : mime->urls())
        out.push_back(rstr(u.toString(QUrl::FullyEncoded)));
    return out;
}

bool clipboard_is_cut()
{
    const QMimeData* mime = QGuiApplication::clipboard()->mimeData();
    if (!mime)
        return false;
    if (mime->hasFormat("x-special/gnome-copied-files"))
        return mime->data("x-special/gnome-copied-files").startsWith("cut");
    return mime->data("application/x-kde-cutselection") == "1";
}

rust::String mime_type(rust::Str path, bool by_content)
{
    static const QMimeDatabase db;
    const auto mode = by_content ? QMimeDatabase::MatchDefault : QMimeDatabase::MatchExtension;
    return rstr(db.mimeTypeForFile(qstr(path), mode).name());
}

rust::String mime_comment(rust::Str mime)
{
    static const QMimeDatabase db;
    return rstr(db.mimeTypeForName(qstr(mime)).comment());
}

}
