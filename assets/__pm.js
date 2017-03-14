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
  DOM.addEventListener('focus', function (event) {
    if (event.target.id === '__pm__commit__message') {
      var changes = DOM.querySelector('#__pm__commit__changes');
      changes.classList.remove('open');
      stickToBottom();
    }
  }, true);
  DOM.addEventListener('click', function (event) {
    var parentElement = event.target.parentElement;
    if (parentElement && parentElement.id === '__pm__commit__changes') {
      parentElement.classList.toggle('open');
      stickToBottom();
    }
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
      root = document.createElement('div');
      root.style.position = 'absolute'; root.style.textAlign = 'center';
      root.style.lineHeight = '3em';
      root.style.left = 0; root.style.right = 0; root.style.bottom = 0;
      root.style.backgroundColor='#fe6d39'; root.style.color="#FFF";
      root.textContent = "An error occurred initializing Superconductor";
    }
    if (open) root.classList.add('open');
    DOM.appendChild(root);
    stickToBottom();
  }

  function stickToBottom() {
    var commit = DOM.querySelector('#__pm__commit');
    if (!commit) return false;
    var stickToBottomCallback = function () {
      commit.scrollTop = commit.scrollHeight;
      window.requestAnimationFrame(stickToBottomCallback);
    };
    setTimeout(function () { stickToBottomCallback = function () {} }, 250);
    stickToBottomCallback();
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

  PM.open();
})(window, PM);
