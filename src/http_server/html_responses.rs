pub const HOME: &str = "\
<h1>Welcome!</h1>
<a href='/wol'>
    <button type='button'>
        <h1>WOL</h1>
    </button>
</a>";
pub const ERROR: &str = "\
<h1>An error occured while handling the query</h1>
<br/>
<a href='/'>
    <button type='button'>
        <h1>HOME</h1>
    </button>
</a>";
pub const WOL_INPUT: &str = "\
<h1>WOL</h1>
<h2>Insert the MAC address of the device to wake</h2>
<form method='get'>
    <label for='mac_addr'>Mac Address:</label>
    <input type='text' id='mac_addr' name='mac_addr'>
    <br/>
    <input type='submit' value='Submit'>
</form>";
pub const WOL_SUCCESS: &str = "\
<h1>WOL</h1>
<h2>Packet sent!</h2>
<br/>
<a href='/'>
    <button type='button'>
        <h1>HOME</h1>
    </button>
</a>";
pub const NOT_ENABLED: &str = "\
<h1>This service is not enabled on this device</h1>
<br/>
<a href='/'>
    <button type='button'>
        <h1>HOME</h1>
    </button>
</a>";
