(PM.superconductor = function(window, PM) {
  var document = window.document;
  var host = document.createElement('div');
  var root = document.createElement('div');
  var DOM;

  setInterval(applyTimeAgo, 60000);

  DOM = (host.attachShadow) ? host.attachShadow({mode: 'open'}) : host;

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
    var form = DOM.querySelector('#__pm__commit');
    if (event.target.type === "submit") {
      serialize(form, event);
      event.preventDefault();
    } else if (event.target.classList.contains('task')) {
      DOM.querySelector("#__pm__commit__task").value = event.target.dataset.name;
      serialize(form, event);
      event.preventDefault();
    } else {
      var legend = closest(event.target, function(e) { return e.id === '__pm__commit__changes_legend'; }, 2);
      if (legend) {
        legend.focus();
        var detailsElement = legend.parentElement;
        var classList = detailsElement.classList;
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
    var debounceRoot = root;
    setTimeout(function() {
      if (root === debounceRoot) serialize(event.target.form, event);
    }, event.target.tagName === "TEXTAREA" ? 250 : 0);
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
      serialize(checkbox.form, event);
    };

    if (isActionable) {
      if (isDetails)  openDetails(parentElement);
      if (isCheckbox) toggleCheckbox(isCheckbox);
    }
  });

  var dragging;
  var dropping;
  DOM.addEventListener('dragstart', function (event) {
    dragging = event.target;
    var dropTargets = DOM.querySelectorAll('.tiles > li');
    for (dropTarget of dropTargets) {
      if (dropTarget === event.target) continue;
      dropTarget.addEventListener('dragenter', dragEnter);
      dropTarget.addEventListener('dragleave', dragLeave);
      dropTarget.addEventListener('dragover', isDropTarget);
      dropTarget.addEventListener('drop', dragDropped);
    };
  });

  DOM.addEventListener('dragend', function (event) {
    dragging = null;
    var dropTargets = DOM.querySelectorAll('.tiles > li');
    for (dropTarget of dropTargets) {
      if (dropTarget === event.target) continue;
      dropTarget.removeEventListener('dragenter', dragEnter);
      dropTarget.removeEventListener('dragleave', dragLeave);
      dropTarget.removeEventListener('dragover', isDropTarget);
      dropTarget.removeEventListener('drop', dragDropped);
    };
  });

  //////////////////////////////////////////////////////////////////////////////////////////////////
  
  function dragEnter(event) {
    if (dragging !== this) {
      this.classList.add('droppable');
      dropping = this;
      event.preventDefault;
    }
  }

  function dragLeave(event) {
    if (dropping !== this) {
      this.classList.remove('droppable');
      event.preventDefault;
    }
  }

  function isDropTarget(event) {
    event.preventDefault();
  };

  function dragDropped(event) {
    this.classList.add('dropped');
    var form = DOM.querySelector('#__pm__commit');
    serialize(form, event);
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

  function serialize(form, event) {
    var serializer = new XMLSerializer();
    var request = document.implementation.createDocument(null, form.name);
    var message = request.children[0];
    var elements = form.elements;
    var filter = Array.prototype.filter.bind(elements);
    var reduce = Array.prototype.reduce.bind(elements);
    elements = filter(function (element) { return element.name !== ""; });
    elements = reduce(function(builder, element) {
        var name = element.name;
        builder[name] = builder[name] || [];
        builder[name].push(element);
        return builder;
    }, {});

    if (event && DOM.activeElement) {
      var focus = request.createElement('focus');
      focus.textContent = DOM.activeElement.id;
      message.appendChild(focus);
    }
    for(name in elements) {
        var inputs = elements[name];
        for (var i = 0; i < inputs.length; i++) {
            var input = inputs[i];
            if (input.name) {
                var element = request.createElement(input.name)
                var inputHasData = Object.keys(input.dataset).length > 0;
                if (inputHasData) {
                    for(data in input.dataset) {
                        var dataElement = request.createElement(data);
                        dataElement.textContent = input.dataset[data];
                        element.appendChild(dataElement);
                    }
                    var valueElement = request.createElement('value');
                    valueElement.textContent = input.value;
                    element.appendChild(valueElement);
                } else {
                    element.textContent = input.value;
                }
                if (input.tagName === 'INPUT' && input.type.toUpperCase() === 'CHECKBOX') {
                    if (input.checked) {
                        message.appendChild(element);
                    }
                } else if (input.tagName === 'BUTTON' || input.tagName === 'INPUT' && input.type.toUpperCase() === 'SUBMIT') {
                    if (event && input === event.target) {
                        var textContent;
                        if (input.value) textContent = input.value;
                        else textContent = event.type;
                        textContent = request.createCDATASection(textContent);
                        element.appendChild(textContent);
                        message.appendChild(element);
                    }
                } else {
                    message.appendChild(element);
                }
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
    applyTimeAgo();
    if (hljs) Array.prototype.forEach.call(root.querySelectorAll('pre code'), hljs.highlightBlock);
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

  function closest(element, filter, limit) {
    limit = limit || -1
    var count = 0;
    while (element) {
      if (filter(element)) return element;
      else if (count === limit) return false;
      else element = element.parentElement;
      count++;
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


  function timeAgo(date) {
    var dateString = date.getAttribute("datetime");
    var timestamp = new Date(dateString).getTime();
    var now = new Date().getTime();
    var distance = timestamp - now;
    var seconds = Math.round(Math.abs(distance) / 1000);
    var minutes = Math.round(seconds / 60);
    var hours = Math.round(minutes / 60);
    var days = Math.round(hours / 24);
    var months = Math.round(days / 30);
    var years = Math.round(days / 365);
    if (seconds < 60) date.innerHTML = "less than a minute ago";
    else if (minutes < 2) date.innerHTML = minutes+" minute ago";
    else if (minutes < 60) date.innerHTML = minutes+" minutes ago";
    else if (hours < 2) date.innerHTML = hours+" hour ago";
    else if (hours < 24) date.innerHTML = hours+" hours ago";
    else if (days < 2) date.innerHTML = days+" day ago";
    else if (days < 30) date.innerHTML = days+" days ago";
    else if (months < 2) date.innerHTML = months+" month ago";
    else if (months < 12) date.innerHTML = months+" months ago";
    else if (years < 2) date.innerHTML = years+" year ago";
    else date.innerHTML = years+' years ago';
  }

  function applyTimeAgo() {
    Array.prototype.forEach.call(root.querySelectorAll('time'), timeAgo)
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
