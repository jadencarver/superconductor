(function(window, PM) {
  var document = window.document;
  var host = document.createElement('div');
  var root = document.createElement('div');
  var DOM = host.attachShadow({mode: 'open'});
  DOM.appendChild(root);
  document.body.appendChild(host);

  var processor = loadProcessor();
  var parser = new DOMParser();
  var socket = openSocket();

  document.addEventListener('keypress', function (event) {
    if (!root) return true;
    if (event.which == 96) { PM.toggle(); }
  });

  //////////////////////////////////////////////////////////////////////////////////////////////////

  function loadProcessor() {
    var processor = new XSLTProcessor();
    var processorRequest = new XMLHttpRequest();
    processorRequest.open("GET", "/__panel.xslt", false);
    processorRequest.send(null);
    processor.importStylesheet(processorRequest.responseXML);
    return processor;
  }

  function openSocket() {
    var socket = new WebSocket("ws://127.0.0.1:2794", "superconductor");
    socket.onmessage = function (event) {
      var state = parser.parseFromString(event.data, "text/xml");
      console.log(state);
      setState(state);
    }
    return socket;
  }

  function setState(state) {
    var open, fragment = processor.transformToFragment(state, document);
    if (root) {
      open = root.classList.contains('open');
      DOM.removeChild(root);
    }
    if (fragment) {
      root = fragment.firstChild;
    } else {
      root = document.createElement('div.error');
      root.textContent = "An Error Occurred";
    }
    if (open) root.classList.add('open');
    DOM.appendChild(root);
  }

  PM.toggle = function () {
    root.classList.toggle('open');
  };

  PM.close = function () {
    root.classList.remove('open');
  };

  PM.open = function () {
    root.classList.add('open');
  };
})(window, PM);
