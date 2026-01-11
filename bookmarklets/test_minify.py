#!/usr/bin/env python3
"""
Tests for minify.py bookmarklet generator.
Run with: python3 test_minify.py
"""

import unittest
import os
import sys
from minify import convert_to_bookmarklet

class TestMinify(unittest.TestCase):
    
    def test_removes_single_line_comments(self):
        """Test that single-line comments are removed."""
        js_code = """
        // This is a comment
        var x = 5;
        var y = 10; // inline comment
        """
        result = convert_to_bookmarklet(js_code)
        self.assertNotIn('//', result)
        self.assertIn('var x = 5', result)
        self.assertIn('var y = 10', result)
    
    def test_removes_multiline_comments(self):
        """Test that multi-line comments are removed."""
        js_code = """
        /* This is a 
           multi-line comment */
        var x = 5;
        """
        result = convert_to_bookmarklet(js_code)
        self.assertNotIn('/*', result)
        self.assertNotIn('*/', result)
        self.assertIn('var x = 5', result)
    
    def test_preserves_urls(self):
        """Test that URLs with // are not broken."""
        js_code = """
        var url = "https://example.com";
        """
        result = convert_to_bookmarklet(js_code)
        self.assertIn('https:', result)
    
    def test_collapses_whitespace(self):
        """Test that multiple spaces are collapsed."""
        js_code = """
        var x    =     5;
        var y = 10;
        """
        result = convert_to_bookmarklet(js_code)
        # Should not have multiple consecutive spaces
        self.assertNotIn('    ', result)
        self.assertNotIn('     ', result)
    
    def test_wraps_in_iife(self):
        """Test that code is wrapped in IIFE."""
        js_code = "var x = 5;"
        result = convert_to_bookmarklet(js_code)
        self.assertTrue(result.startswith('javascript:(function(){'))
        self.assertTrue(result.endswith('})();'))
    
    def test_creates_single_line(self):
        """Test that output is a single line."""
        js_code = """
        var x = 5;
        var y = 10;
        var z = 15;
        """
        result = convert_to_bookmarklet(js_code)
        self.assertNotIn('\n', result)
        self.assertNotIn('\r', result)
    
    def test_preserves_function_logic(self):
        """Test that function logic is preserved."""
        js_code = """
        function greet(name) {
            return "Hello, " + name;
        }
        greet("World");
        """
        result = convert_to_bookmarklet(js_code)
        self.assertIn('function greet', result)
        self.assertIn('return', result)
        self.assertIn('Hello,', result)
        self.assertIn('greet("World")', result)
    
    def test_empty_code(self):
        """Test handling of empty code."""
        js_code = ""
        result = convert_to_bookmarklet(js_code)
        # Should still wrap in IIFE even if empty
        self.assertTrue(result.startswith('javascript:(function(){'))
        self.assertTrue(result.endswith('})();'))
    
    def test_complex_code_with_strings(self):
        """Test that strings with special characters are preserved."""
        js_code = """
        var msg = "User: Hello\\n\\nGemini: Hi there!";
        console.log(msg);
        """
        result = convert_to_bookmarklet(js_code)
        self.assertIn('User:', result)
        self.assertIn('Gemini:', result)
        self.assertIn('console.log', result)

class TestMinifyIntegration(unittest.TestCase):
    """Integration tests that verify actual bookmarklet files work."""
    
    def setUp(self):
        """Set up test by finding the bookmarklets directory."""
        self.script_dir = os.path.dirname(os.path.abspath(__file__))
    
    def test_gemini_js_exists(self):
        """Test that gemini.js exists and can be read."""
        gemini_file = os.path.join(self.script_dir, "gemini.js")
        self.assertTrue(os.path.exists(gemini_file), f"gemini.js not found at {gemini_file}")
        
        with open(gemini_file, 'r') as f:
            content = f.read()
        self.assertGreater(len(content), 0)
        self.assertIn('scrapeGemini', content)
    
    def test_aim_js_exists(self):
        """Test that aim.js exists and can be read."""
        aim_file = os.path.join(self.script_dir, "aim.js")
        self.assertTrue(os.path.exists(aim_file), f"aim.js not found at {aim_file}")
        
        with open(aim_file, 'r') as f:
            content = f.read()
        self.assertGreater(len(content), 0)
        self.assertIn('scrapeAim', content)
    
    def test_chat_js_exists(self):
        """Test that chat.js exists and can be read."""
        chat_file = os.path.join(self.script_dir, "chat.js")
        self.assertTrue(os.path.exists(chat_file), f"chat.js not found at {chat_file}")
        
        with open(chat_file, 'r') as f:
            content = f.read()
        self.assertGreater(len(content), 0)
        self.assertIn('location.href', content)
    
    def test_gemini_bookmarklet_generation(self):
        """Test that gemini bookmarklet can be generated."""
        gemini_file = os.path.join(self.script_dir, "gemini.js")
        with open(gemini_file, 'r') as f:
            content = f.read()
        
        result = convert_to_bookmarklet(content)
        self.assertTrue(result.startswith('javascript:'))
        self.assertIn('scrapeGemini', result)
        self.assertIn('downloadGeminiFile', result)
    
    def test_aim_bookmarklet_generation(self):
        """Test that aim bookmarklet can be generated."""
        aim_file = os.path.join(self.script_dir, "aim.js")
        with open(aim_file, 'r') as f:
            content = f.read()
        
        result = convert_to_bookmarklet(content)
        self.assertTrue(result.startswith('javascript:'))
        self.assertIn('scrapeAim', result)
        self.assertIn('downloadAimFile', result)

if __name__ == '__main__':
    # Run tests with verbose output
    unittest.main(verbosity=2)
