(function(window, PM) {
  var panel;
  var panelSpeed = 500;
  document.addEventListener('keypress', function (event) {
    if (event.which == 96) { PM.toggle(); }
  });

  PM.toggle = function () {
    if (panel) PM.close();
    else PM.open();
  };

  PM.close = function () {
    panel.style.maxHeight = '0';
    setTimeout(function() {
      document.body.removeChild(panel);
      panel = false;
    }, panelSpeed);
  };

  PM.open = function () {
    if (panel) return true;
    panel = document.createElement('div');
    panel.setAttribute('id', '__pm_panel');
    panel.setAttribute('style', ''
                       + 'box-sizing: border-box;'
                       + 'background-color: #000000;'
                       + 'color: #FFFFFF;'
                       + 'position: fixed;'
                       + 'bottom: 0;'
                       + 'left: 0;'
                       + 'right: 0;'
                       + 'line-height: 1rem;'
                       + 'transition: max-height '+panelSpeed+'ms;'
                      );
    document.body.appendChild(panel);
    panel.innerHTML = (''
    + '<div style="margin: 5pt; font-weight: 400;">Project Management</div>'
    + '<ul style="margin: 0; font-size: 0.8em; list-style-type: none; display: flex; padding: 0;">'
    +   '<li style="width: 100pt; height: 100pt; background-color: #666666; padding: 5pt; margin: 5pt;">'
    +     '<strong>'+PM.current.name+'</strong>'
    +   '</li>'
    +   '<li style="width: 100pt; height: 100pt; background-color: #666666; padding: 5pt; margin: 5pt;">'
    +     '<strong>'+PM.current.name+'</strong>'
    +   '</li>'
    +   '<li style="width: 100pt; height: 100pt; background-color: #666666; padding: 5pt; margin: 5pt;">'
    +     '<strong>'+PM.current.name+'</strong>'
    +   '</li>'
    + '</ul>'
    );
    height = panel.offsetHeight;
    panel.style.maxHeight = '0';
    setTimeout(function () { panel.style.maxHeight = height+'px'; }, 1);
  };
})(window, PM);
