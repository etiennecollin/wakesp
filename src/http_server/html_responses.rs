pub const HOME: &[u8] = b"";

pub const WOL_INPUT: &[u8] = b"\
<h1>WOL</h1>
<p>Insert the MAC address of the device to wake</p>
<form method=\"get\">
  <div>
    <label for=\"mac_addr\">Mac Address:</label>
    <input type=\"text\" id=\"mac_addr\" name=\"mac_addr\" />
  </div>
  <input type=\"submit\" value=\"Submit\" />
</form>";

pub const WOL_SUCCESS: &[u8] = b"\
<h1>WOL</h1>
<p>Packet sent!</p>";

pub const ERROR: &[u8] = b"\
<h1>Error</h1>
<p>An error occured while handling the query</p>";

pub const NOT_ENABLED: &[u8] = b"\
<h1>Error</h1>
<p>This service is not enabled on this device</p>";

pub const HTML_MENU: &[u8] = b"\
\r\n<br />
<ol>
  <h1>Menu</h1>
  <li>
    <a class=\"arrow\" href=\"/wol\"
      ><i class=\"fas fa-arrow-alt-right\"></i>WOL</a
    >
  </li>
</ol>\r\n";

pub const HTML_HEADER: &[u8] = b"\
<!doctype html>
<html lang=\"en\">
<head>
<meta charset=\"UTF-8\" />
<title>Wakesp</title>
<meta name=\"description\" content=\"Wakesp\" />
<meta name=\"color-scheme\" content=\"dark\" />
<link
  rel=\"stylesheet\"
  href=\"https://pro.fontawesome.com/releases/v5.15.4/css/all.css\"
/>
<meta
  name=\"viewport\"
  content=\"width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0\"
/>
<style>
  @import \"https://fonts.googleapis.com/css?family=Press%20Start%202P\";

  * {
    margin: 0;
    padding: 0;
  }

  html,
  body {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: center;
    height: 100%;
    background-color: #221;
  }

  html,
  body,
  input {
    font-family: \"Press Start 2P\";
    font-size: 1rem;
    line-height: 2;
    text-transform: uppercase;
  }

  p,
  label {
    color: #f6eb14;
    padding: 0.5em;
    margin: 0.5em;
  }

  li {
    color: #4faf44;
  }

  a,
  input {
    padding: 0.5em;
    margin: 0.5em;
    background-color: #4faf44;
    border: 1px solid #f6eb14;
    border-radius: 0.5em;
    text-decoration: none;
    text-align: center;
    color: #fff090;
  }

  form {
    display: flex;
    flex-direction: column;
    vertical-align: middle;
    justify-content: center;
    align-items: center;
    text-align: center;
    background-color: #ffe1;
    border: 1px solid #f6eb14;
    border-radius: 0.5em;
    padding-bottom: 0.5em;
  }

  .arrow {
    position: relative;
    display: block;
    color: #f6eb14;
  }

  .arrow i {
    display: none;
  }

  .arrow:hover,
  input[type=\"submit\"]:hover {
    cursor: pointer;
    background-color: transparent;
  }

  .arrow:hover i {
    display: block;
    position: absolute;
    top: 1em;
    left: -6em;
  }
</style>
</head>\r\n";

pub const HTML_TAIL: &[u8] = b"\r\n</body>\r\n</html>\r\n";
