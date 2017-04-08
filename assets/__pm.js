(function(window, PM) {
  var document = window.document;
  var host = document.createElement('div');
  var root = document.createElement('div');
  var DOM;
  if (host.attachShadow) {
    DOM = host.attachShadow({mode: 'open'});
  } else {
    DOM = host;
  }
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
      if (changes) changes.classList.remove('open');
      stickToBottom();
    }
  }, true);

  DOM.addEventListener('click', function (event) {
    if (event.target.type === "submit") {
      var form = DOM.querySelector('#__pm__commit');
      sendForm(form, event);
      event.preventDefault();
    } else {
      var parentElement = event.target.parentElement;
      if (parentElement && parentElement.id === '__pm__commit__changes') {
        var classList = parentElement.classList;
        if (classList.contains('open')) {
            classList.remove('open');
            DOM.querySelector('#__pm__commit__message').focus();
        } else {
            classList.add('open');
        }
        stickToBottom();
      }
    }
  });

  DOM.addEventListener('change', function (event) {
    sendForm(event.target.form, event);
  });

  DOM.addEventListener('keyup', function (event) {
    var parentElement = event.target.parentElement;
    var EnterKey = 13, SpaceKey = 32;
    var isActionable = event.which === EnterKey || event.which === SpaceKey;
    var isCheckbox = event.target.querySelector('input[type=checkbox]');
    var isDetails = parentElement && parentElement.classList.contains('details');

    var openDetails = function (details) {
      details.classList.toggle('open');
      stickToBottom();
    };
    var toggleCheckbox = function (checkbox) {
      checkbox.checked = !checkbox.checked;
      sendForm(checkbox.form, event);
    };

    if (isActionable) {
      if (isDetails)  openDetails(parentElement);
      if (isCheckbox) toggleCheckbox(isCheckbox);
    }
  });

  var dragging;
  DOM.addEventListener('dragstart', function (event) {
    dragging = event.target;
    var dropTargets = DOM.querySelectorAll('.tiles > li');
    for (dropTarget of dropTargets) {
      if (dropTarget === event.target) continue;
      dropTarget.addEventListener('dragenter', dragEnter);
      dropTarget.addEventListener('dragleave', dragLeave);
      dropTarget.addEventListener('dragover', isDropTarget);
      dropTarget.addEventListener('drop', isDropTarget);
    };
  });
  DOM.addEventListener('dragend', function (event) {
    var dropTargets = DOM.querySelectorAll('.tiles > li');
    for (dropTarget of dropTargets) {
      if (dropTarget === event.target) continue;
      dropTarget.removeEventListener('dragenter', dragEnter);
      dropTarget.removeEventListener('dragleave', dragLeave);
      dropTarget.removeEventListener('dragover', isDropTarget);
      dropTarget.removeEventListener('drop', isDropTarget);
    };
  });

  //////////////////////////////////////////////////////////////////////////////////////////////////
  
  function dragEnter(event) {
    if (dragging !== this) {
      this.classList.add('droppable');
      event.preventDefault;
    }
  }
  function dragLeave(event) {
    this.classList.remove('droppable');
    event.preventDefault;
  }

  function isDropTarget(event) {
    event.preventDefault();
  };


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

  function sendForm(form, event) {
    var serializer = new XMLSerializer();
    var request = document.implementation.createDocument(null, form.name);
    var message = request.children[0];
    var inputs = form.elements;
    if (event) {
      var focus = request.createElement('focus');
      focus.textContent = event.target.id;
      message.appendChild(focus);
    }
    for (var i = 0; i < inputs.length; i++) {
      var input = inputs[i];
      if (input.name) {
        var element = request.createElement(input.name)
        element.textContent = input.value;
        if (input.tagName === 'INPUT' && input.type.toUpperCase() === 'CHECKBOX') {
          if (input.checked) {
            message.appendChild(element);
          }
        } else if (input.tagName === 'INPUT' && input.type.toUpperCase() === 'SUBMIT') {
          if (event && input === event.target) {
            element.textContent = event.type;
            message.appendChild(element);
          }
        } else {
          message.appendChild(element);
        }
      }
    }
    console.log(request);
    socket.send(serializer.serializeToString(request));
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
      root.style.position = 'fixed';
      root.style.left = 0; root.style.right = 0; root.style.bottom = 0;
      root.style.textAlign = 'center'; root.style.lineHeight = '3em';
      root.style.backgroundColor='#fe6d39'; root.style.color="#fff";
      root.textContent = "An error occurred initializing Superconductor";
    }
    DOM.appendChild(root);
    if (open) {
      root.classList.add('open');
      var focus = state.querySelector('focus');
      if (focus) var focusId = focus.textContent;
      if (focusId) var focusElement = DOM.querySelector('#'+focusId);
      if (focusElement) {
        var detailsElement = closest(focusElement, function (e) {
          return e.classList.contains('details');
        });
        if (detailsElement) detailsElement.classList.add('open');
        focusElement.classList.add('no-transition');
        focusElement.focus();
        focusElement.classList.remove('no-transition');
      }
    }
    stickToBottom();
  }

  function closest(element, filter) {
    while (element) {
      if (filter(element)) return element;
      else element = element.parentElement;
    }
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
