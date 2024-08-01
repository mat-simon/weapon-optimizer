import { NextResponse } from 'next/server'

export async function POST(request: Request) {
  const body = await request.json()
  
  const response = await fetch('http://127.0.0.1:8080/optimize', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(body),
  })

  const result = await response.json()
  return NextResponse.json(result)
}