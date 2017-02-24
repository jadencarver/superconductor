require 'pry'
require 'rack/request'
require 'rack/response'
require 'superconductor'

class Superconductor::Documentation
  include Rack::Utils

  def initialize(path: nil, index: "index.html")
    @path ||= File.join(Dir.getwd, 'doc')
    @index = index
  end

  def call(env)
    path = env["PATH_INFO"][1..-1]
    file = path == "" ? @index : path
    headers = HeaderHash.new({
      'Content-Type' => content_type(file)
    })
    body = [open(File.join(@path, file)).read]
    status = 200
    [status, headers, body]
  end

  private def content_type(file)
    return "text/html"        if file =~ /.html?$/
    return "text/css"         if file =~ /.css$/
    return "text/javascript"  if file =~ /.js$/
    return "text/plain"
  end
end
