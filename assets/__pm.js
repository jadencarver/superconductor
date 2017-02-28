(function(window, PM) {
  var panel;
  var panelElement;
  var panelSpeed = 500;

  var processorRequest = new XMLHttpRequest();
  var processor = new XSLTProcessor();
  var state = document.implementation.createDocument("", "", null);

  // Fake Data
  root = state.createElement('state');
  state.appendChild(root);
  ident = state.createElement('ident');
  ident.textContent = "Jaden";
  root.appendChild(ident);
  //\\

  processorRequest.open("GET", "/__panel.xslt", false);
  processorRequest.send(null);
  processor.importStylesheet(processorRequest.responseXML);

  document.addEventListener('keypress', function (event) {
    if (event.which == 96) { PM.toggle(); }
  });

  var socket = new WebSocket("ws://127.0.0.1:2794", "superconductor");
  socket.onmessage = function (event) {
    state = event.data;
    render();
  }

  function render() {
    var fragment = processor.transformToFragment(state, document);
    if (fragment) {
      if (panel && panel.parentElement) {
        document.body.removeChild(panel);
      }
      panel = document.createElement('div');
      var shadow = panel.attachShadow({mode: 'open'});
      panelElement = fragment.firstChild;
      shadow.appendChild(panelElement);
      document.body.appendChild(panel);
    }
  }

  PM.toggle = function () {
    if (panel) PM.close();
    else PM.open();
  };

  PM.close = function () {
    if (!panel) return false
      panelElement.style.maxHeight = '0';
    setTimeout(function() {
      if (!panel || !panel.parentElement) return false
        document.body.removeChild(panel);
      panel = false;
    }, panelSpeed);
  };

  PM.open = function () {
    if (panel) return true;
    render();
    height = panelElement.offsetHeight;
    panelElement.style.maxHeight = '0';
    setTimeout(function () {
      panelElement.classList.add('transition');
      panelElement.style.maxHeight = height+'px';
    }, 1);
  };
})(window, PM);
