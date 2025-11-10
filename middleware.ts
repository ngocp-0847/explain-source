import { NextResponse } from 'next/server'
import type { NextRequest } from 'next/server'

export function middleware(request: NextRequest) {
  // Get token from localStorage (client-side only, so we check cookie instead)
  const authCookie = request.cookies.get('auth-storage')
  
  // Public paths that don't require authentication
  const publicPaths = ['/login', '/register']
  const isPublicPath = publicPaths.some(path => request.nextUrl.pathname.startsWith(path))

  // If accessing login/register while authenticated, redirect to homepage
  if (isPublicPath && authCookie) {
    try {
      const authData = JSON.parse(authCookie.value)
      if (authData?.state?.isAuthenticated) {
        return NextResponse.redirect(new URL('/', request.url))
      }
    } catch (e) {
      // Invalid cookie, continue to login
    }
  }

  // If accessing protected paths without auth, redirect to login
  if (!isPublicPath && !authCookie) {
    return NextResponse.redirect(new URL('/login', request.url))
  }

  // Check if authenticated
  if (!isPublicPath) {
    try {
      const authData = authCookie ? JSON.parse(authCookie.value) : null
      if (!authData?.state?.isAuthenticated) {
        return NextResponse.redirect(new URL('/login', request.url))
      }
    } catch (e) {
      // Invalid cookie, redirect to login
      return NextResponse.redirect(new URL('/login', request.url))
    }
  }

  return NextResponse.next()
}

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     */
    '/((?!api|_next/static|_next/image|favicon.ico).*)',
  ],
}

