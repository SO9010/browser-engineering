pub mod html;
pub mod layout;
pub mod renderer;
pub mod url;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Error no site given!!!\nUsage: {} <url1> <url2> ...",
            args[0]
        );
        std::process::exit(1);
    }
    for arg in args.iter().skip(1) {
        if let Err(e) = url::load(arg) {
            eprintln!("Error loading URL {}: {}", arg, e);
        }
    }
}

/*
HTTP/1.1 200 OK
\r\nServer: nginx/1.18.0 (Ubuntu)
\r\nDate: Mon, 26 May 2025 15:07:50 GMT
\r\nContent-Type: text/html
\r\nContent-Length: 5124
\r\nLast-Modified: Wed, 22 Mar 2023 14:54:48 GMT
\r\nConnection: keep-alive
\r\nETag: \"641b16b8-1404\"
\r\nReferrer-Policy: strict-origin-when-cross-origin
\r\nX-Content-Type-Options: nosniff
\r\nFeature-Policy: accelerometer 'none'; camera 'none'; geolocation 'none'; gyroscope 'none'; magnetometer 'none'; microphone 'none'; payment 'none'; usb 'none'
\r\nContent-Security-Policy: default-src 'self'; script-src cdnjs.cloudflare.com 'self'; style-src cdnjs.cloudflare.com 'self' fonts.googleapis.com 'unsafe-inline'; font-src fonts.googleapis.com fonts.gstatic.com cdnjs.cloudflare.com; frame-ancestors 'none'; report-uri https://scotthelme.report-uri.com/r/d/csp/enforce
\r\nAccept-Ranges: bytes
\r\n
\r\n<!DOCTYPE HTML>
\r\n<html>
\r\n\t<head>
\r\n\t\t<title>HTTP Forever</title>
\r\n\t\t<meta http-equiv=\"content-type\" content=\"text/html; charset=utf-8\" />
\r\n\t\t<meta name=\"description\" content=\"A site that will always be available over HTTP!\" />
\r\n\t\t<meta name=\"keywords\" content=\"HTTP WiFi Captive Portal\" />
\r\n\t\t<!--[if lte IE 8]><script src=\"css/ie/html5shiv.js\"></script><![endif]-->
\r\n\t\t<script src=\"https://cdnjs.cloudflare.com/ajax/libs/jquery/3.3.1/jquery.min.js\" integrity=\"sha256-FgpCb/KJQlLNfOu91ta32o/NMZxltwRo8QtmkMRdAu8=\" crossorigin=\"anonymous\"></script>
\r\n\t\t<script src=\"https://cdnjs.cloudflare.com/ajax/libs/skel/3.0.1/skel.min.js\" integrity=\"sha
*/
